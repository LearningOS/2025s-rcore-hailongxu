# CH3 功能实现
trace_request  
为0和1的时候，进行id变化成地址，直接进行读写操作

为2的时候，TaskControlBlock在任务控制块中，创建一个数组，保存各个调用的次数存储元素(syscall_id,count)，发生调用时，进行查找，count加1

初始化时都syscall_id，只包括ch3用到的，SYSCALL_WRITE, SYSCALL_EXIT, SYSCALL_YIELD, SYSCALL_GET_TIME, SYSCALL_TRACE 这5个id，count部分初始化为0

# CH3 简答作业

## 第1题
```plaintext
[rustsbi] RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|
[rustsbi] Implementation     : RustSBI-QEMU Version 0.2.0-alpha.2
[rustsbi] Platform Name      : riscv-virtio,qemu
[rustsbi] Platform SMP       : 1
[rustsbi] Platform Memory    : 0x80000000..0x88000000
[rustsbi] Boot HART          : 0
[rustsbi] Device Tree Region : 0x87000000..0x87000ef2
[rustsbi] Firmware Address   : 0x80000000
[rustsbi] Supervisor Address : 0x80200000
[rustsbi] pmp01: 0x00000000..0x80000000 (-wr)
[rustsbi] pmp02: 0x80000000..0x80200000 (---)
[rustsbi] pmp03: 0x80200000..0x88000000 (xwr)
[rustsbi] pmp04: 0x88000000..0x00000000 (-wr)
[kernel] Hello, world!
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] Panicked at src/task/mod.rs:142 All applications completed!
```

我所用的SBI的版本是：RustSBI version 0.3.0-alpha.2,

ch2b_bad_address.rs 的错误输出：
```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
```

ch2b_bad_instructions.rs 的错误输出：
```
[kernel] IllegalInstruction in application, kernel killed it.
```

ch2b_bad_register.rs 的错误输出：
```
[kernel] IllegalInstruction in application, kernel killed it.
```

## 第2题
### 1
刚进入 __restore 时，sp 代表了什么值。请指出 __restore 的两种使用情景。
# case1: start running app by __restore，开始运行app的时候
# case2: back to U after handling trap，返回用户态的时候

### 2
```asm
ld t0, 32*8(sp) # sp
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```
恢复如下寄存器：
sstatus 状态寄存器，标识是S态还是U态  
sepc 发生异常时的返回地址  
sscratch 执行用户栈顶  
为返回用户态做准备

### 3 
Q: L50-L56：为何跳过了 x2 和 x4？

x2 用来存放的用户栈顶，已经放在了 sscratch中了  
x4 App中没有用到，忽略了

### 4
Q: L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？
csrrw sp, sscratch, sp

sp 此时指向用户栈顶，这样用户态的程序能继续运行了  
sscratch 此时指向内核栈顶，給保存起来了

### 5
Q: __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

L61    sret  
sstauts的状态会被重置为用户态
pc也会指向，sepc的地址

### 6
Q: L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

sp指向内核栈顶  
sscratch指向用户栈顶  
进行一次交换，因为已经陷入S态了，sp首要指向内核栈，保证栈正确性

### 7
Q: 从 U 态进入 S 态是哪一条指令发生的？

syscall 函数  
具体是 调用 ecall 指令时



# 荣誉准则
1 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

>《你交流的对象说明》  
都是在微信中交流的：
和 助教 陈宏毅：讨论了 “这个为啥非要拷贝到dst呢，直接用src”
>
>和 助教 殷金钰，陈宏毅，及学员 乍得 Jiang Sheng，实现代码题意本身进行了交流

2 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

>《你参考的资料说明》  
https://learningos.cn/rCore-Camp-Guide-2025S/chapter3
都在这些文档中，其余零星的在网上搜了些riscv汇编指令用法

3 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

# 反馈
你对本次实验设计及难度/工作量的看法，以及有哪些需要改进的地方，欢迎畅所欲言

1  如何提交实验结果，在哪  
2 Reports中的文件，是一个是还是多个，和作业内的如何对应，这样能减少非技术性的沟通  
3 难度适中，挺好，因为很多人包括我，是没有接触过OS开发的，头次接触，得有一段时间的适应性。