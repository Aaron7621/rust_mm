//! Implementation of [`FrameAllocator`] which 
//! controls all the frames in the operating system.

use alloc::boxed::Box;
use super::{PhysAddr, PhysPageNum};
use crate::config::MEMORY_END;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;
use crate::get_time;
use crate::mm::config;

/// manage a frame
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}


impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}



/// an implementation for frame allocator using a bitmap
pub struct BitMapFrameAllocator {
    bitmap: Vec<bool>,
    start_ppn: usize,
    end_ppn: usize,
}

impl BitMapFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.start_ppn = l.0;
        self.end_ppn = r.0;
        let length:usize = self.end_ppn - self.start_ppn;
        for _i in 0..length {
            self.bitmap.push(false);
        }

    }

    pub fn visible(&self) {
        println!("[BitMapFrameAllocator] bitmap:");
        let element_per_row = 70;
        let mut count = 1;
        for bitflags in &self.bitmap {
            print!("{}", if *bitflags {1} else {0});
            if count % element_per_row == 0 { print!("\n"); }
            count += 1;
        }
        println!("\n[BitMapFrameAllocator] bitmap ended");
    }

    pub fn remaining(&self) -> usize {
        self.bitmap.iter().filter(|&n| *n == false).count()
    }
}
impl FrameAllocator for BitMapFrameAllocator {
    fn new() -> Self {
        Self{
            bitmap: Vec::new(),
            start_ppn: 0,
            end_ppn: 0,
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum>{
        let result = self.bitmap.iter().position(|&r| r==false);

        if let Some(index) = result {
            // println!("index={}", index);
            self.bitmap[index] = true;
            Some((index + self.start_ppn).into())
        }else {
            None
        }
    }

    fn dealloc(&mut self, ppn:PhysPageNum){
        let index = ppn.0 - self.start_ppn;
        let bit = self.bitmap[index];
        if bit == false { panic!("Frame ppn={:#x} has not been allocated!", ppn.0); }
        else{
            self.bitmap[index] = false;
        }
    }

}


/// an implementation for frame allocator using a stack
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
        // self.visible();
    }

    pub fn remaining(&self) -> usize {
        self.recycled.len() + self.end - self.current
    }

    pub fn visible(&self) {
        println!("[StackFrameAllocator] current={}, end={}", self.current, self.end);
        println!("[StackFrameAllocator] recycled stack:");
        for ppn in &self.recycled {
            println!("    {}", ppn);
        }
        println!("[StackFrameAllocator] recycled stack ended");
    }

    pub fn get_size(&self) -> usize {
        self.recycled.len() * 8
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}


/// A FrameAllocator using a LinkedList
pub struct LinkedListAllocator {
    list: List<usize>,
}

impl LinkedListAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        for i in l.0..r.0 {
            self.list.push(i);
        }
        // self.visible();
    }
    pub fn remaining(&self) -> usize {
        self.list.len()
    }

    pub fn visible(&self){
        let mut next = &self.list.next;
        println!("[LinkedListAllocator] Linked list:");
        loop {
            match next {
                Some(node) => {
                    print!("{} -> ", node.value);
                    next = &node.next;
                }
                None => {
                    print!("None");
                    break;
                }
            }
        }

        println!("\n[LinkedListAllocator] Linked list ended");
    }

}
impl FrameAllocator for LinkedListAllocator {
    fn new() -> Self {
        Self {
            list: List::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        let result = self.list.get_last_value();
        match result {
            Some(page) => {
                let ppn = *page;
                // println!("In frame_allocator line 183, ppn allocated:{}", ppn);

                self.list.pop();
                Some(ppn.into())
            }
            None => { None }
        }
    }

    fn dealloc(&mut self, ppn:PhysPageNum) {
        self.list.push(ppn.0);
    }
}


/// Define a List
type NextNode<T> = Option<Box<Node<T>>>;

#[derive(Clone, Debug)]
struct Node<T> {
    value: T,
    next: NextNode<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Self {
        Node {
            value: elem,
            next: None,
        }
    }

    fn set_next(&mut self, node: Option<Self>) {
        self.next = None;
        if let Some(x) = node {
            self.next = Some(Box::new(x));
        }
    }

    fn get_next<'a>(&'a mut self) -> Option<&'a mut Self> {
        if let Some(ref mut x) = self.next {
            return Some(x);
        }
        None
    }

    fn get<'a>(&'a mut self, index: usize) -> Option<&'a mut Self> {
        if index == 0 {
            return Some(self);
        }
        if let Some(x) = self.get_next() {
            x.get(index - 1)
        } else {
            None
        }
    }

    fn get_last<'a>(&'a mut self) -> &'a mut Self {
        if let Some(ref mut x) = self.next {
            return x.get_last();
        }
        self
    }

    fn get_last_immutable<'a>(&'a self) -> &'a Self {
        if let Some(ref x) = self.next {
            return x.get_last_immutable();
        }
        self
    }

    fn get_value(&self) -> &T {
        &self.value
    }

    fn push(&mut self, elem: T) {
        let new_node = Node::new(elem);
        self.get_last().set_next(Some(new_node));
    }
}


#[derive(Clone, Debug)]
pub struct List<T> {
    len: usize,
    next: NextNode<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { len: 0, next: None }
    }

    fn get_next<'a>(&'a mut self) -> Option<&'a mut Node<T>> {
        if let Some(ref mut x) = self.next {
            return x.get_next();
        }
        None
    }

    fn get<'a>(&'a mut self, index: usize) -> Option<&'a mut Node<T>> {
        if index > self.len || index == 0 {
            return None;
        }

        let node = self.get_next().unwrap();
        if index == 1 {
            return Some(node);
        }

        node.get(index - 1)
    }

    fn get_last<'a>(&'a mut self) -> Option<&'a mut Node<T>> {
        if let Some(ref mut x) = self.next {
            Some(x.get_last())
        } else {
            None
        }
    }

    fn get_last_immutable<'a>(&'a self) -> Option<&'a Node<T>> {
        if let Some(ref x) = self.next {
            Some(x.get_last_immutable())
        } else {
            None
        }
    }

    pub fn get_last_value(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        Some(self.get_last_immutable().unwrap().get_value())
    }

    pub fn push(&mut self, elem: T) {
        if self.len == 0 {
            self.next = Some(Box::new(Node::new(elem)));
        } else {
            if let Some(ref mut x) = self.get_last() {
                x.push(elem);
            }
        }
        self.len += 1;
    }

    pub fn pop(&mut self) {
        if self.len == 0 {
            return ();
        }
        self.len -= 1;
        let index = self.len;
        self.get(index - 1).unwrap().set_next(None);
    }

    pub fn len(&self) -> usize {
        self.len
    }
}




lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: UPSafeCell<config::FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(config::FrameAllocatorImpl::new()) };
}

/// initiate the frame allocator
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    // let time_1 = get_time();
    // let allocator_size = FRAME_ALLOCATOR.exclusive_access().get_size();
    let frame_tracker = FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new);
    // let time_2 = get_time();
    // println!("[frame_allocator] allocated a frame, time cost={}", time_2-time_1);

    frame_tracker

}

/// deallocate a frame
fn frame_dealloc(ppn: PhysPageNum) {
    // let time_1 = get_time();

    // dealloc
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
    // let time_2 = get_time();
    // println!("[frame_allocator] deallocated a frame, time cost={}", time_2-time_1);

}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    println!("---frame allocator test started---");
    extern "C" {
        fn ekernel();
    }
    // println!("[frame_allocator_test] ekernel={}" ,ekernel as usize);
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    println!("[frame_allocator_test]After being allocated, frames state:");
    FRAME_ALLOCATOR.exclusive_access().visible();
    println!("[frame_allocator_test] Now clear the vector, frame state:");
    v.clear();
    FRAME_ALLOCATOR.exclusive_access().visible();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("---frame allocator test passed!---");
}
