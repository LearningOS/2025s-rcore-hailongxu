# CH5 功能实现
spawn参考initproc的new过程，本质上是把fork和exec给综合起来，把多余的重复的动作去掉

stride的实现，关键点在于比较大小的时候，不能用大于号进行直接比较，这里处理了溢出问题，必须让两个stride相减，然后转成有符号数，进行判断大小，要不然，流程就错乱了


# CH5 问答作业

## 第1题 stride 算法深入
stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。
### 1
实际情况是轮到 p1 执行吗？为什么？

`pass=BIG_STRIDE/P->priority`，p2.stride+10=265，此时产生溢出，265-255=10，小于p1.stride=255，所以p2执行 


### 2
我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。  
为什么？尝试简单说明（不要求严格证明）

证明：设Pmax.stride最大，Pmin.stride最小，此时下一步，Pmin.stride += Pmin.pass，这时分两种情况讨论

1. Pmax.stride > Pmin.stride+Pmin.pass，若持续这样，就会不断迭代到STRIDE_MAX – STRIDE_MIN <= BigStride / max{P.pass} 
1. Pmax.stride < Pmin.stride+Pmin.pass，这种情况，假设Pa.stride最小，Pmin.stride+Pmin.pass - Pa.stride = Pmin.stride-Pa.stride + Pmin.pass，我们知道，Pmin.stride-Pa.stride < 0，所以此时 STRIDE_MAX – STRIDE_MIN < Pmin.pass = Max{P.pass}
1. 又由于P.pass = BigStride/P.priority，由于 P.priority>=2 故 STRIDE_MAX – STRIDE_MIN <= BigStride/2

### 3 
已知以上结论，考虑溢出的情况下，设计比较器
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let r = self.stride - other.stride;
        let r = r as i8;
        r.cmp(0)
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        self.stride == other.stride
    }
}
```


# 荣誉准则
1 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

>《你交流的对象说明》  
都是在微信中交流的：
和 学员 SS 李昌荣 Dako：讨论了 “run-tasks的loop循环和schedule的问题”

2 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

>《你参考的资料说明》  
https://learningos.cn/rCore-Camp-Guide-2025S/chapter5
有关stride的算法参考了如下：https://blog.csdn.net/u012750235/article/details/131884423

3 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

# 反馈
你对本次实验设计及难度/工作量的看法，以及有哪些需要改进的地方，欢迎畅所欲言

1 本章有些难度，导致花了一些时间，不过要也要赞一个，有挑战才有进步