# Memory Management Module based on Rust

This is my graduation project in my undergraduate study. 

I learn about this rust-based os in a course named r-Core, where I learnt about the principle of certain core concept in operating system, including trap, system call, page table, etc. 

Express my gratitude to the rCore's contributors and their work:

https://rcore-os.cn/rCore-Tutorial-Book-v3/



My main work is in os/mm, where I modify and extend the original memory management method.

- I add more frame allocators based on different algorithms and data structure.
- I apply a new layer, the segment layer, based on the frame allocator, that is able to allocate a continuous block of memory to the process.

A unified interface is defined for the of memory management modules, and other modules can transparently use the allocating interface under different allocators.

And I use the method of conditional compilation to compile the kernel, that we can choose a certain allocator in the os.

I also upload my graduation paper of this project. Please feel free to criticize and discuss with me, about anything in this project. 



# 基于 Rust 的内存管理模块

这是我的本科毕业设计。

我在一个名为 r-Core 的课程中了解了这个基于 rust 的操作系统，通过学习该课程我了解了操作系统中一些核心的原理，包括陷阱、系统调用、页表等。

对 rCore 的贡献者及其工作表示真诚的感谢：

https://rcore-os.cn/rCore-Tutorial-Book-v3/



我的主要工作是在 os/mm 中，我对原来的内存管理方法进行了修改和扩展：

- 添加了基于不同算法和数据结构的页帧分配器
- 基于页帧分配器应用了一个新层，即分段层，它能够为进程分配连续的内存块

定义了统一的接口进行内存模块的开发，其余模块可以透明地使用不同分配器。

使用条件编译的方法来编译内核，我们可以选择操作系统基于某个分配器运行。

毕设论文也上传在仓库中。欢迎批评和讨论！
