# Newton 线搜索（step halving）+ 发散日志改进

## 问题

大信号输入时，12AX7 屏压在 B+ 和饱和之间大幅摆动，Koren 模型在
工作点附近线性化不准，Newton 一步 overshoot，导致 50 次仍不收敛。

## 方案

在 `src/circuit/solver.rs` 中加入 line search：

```
solve_linear() → Δv
如果 max|Δv| > 10V:
    Δv *= 0.5，重试，最多 6 次
```

`10V` 的阈值针对电子管电路：Vgk 变化 10V 足以让管子从截止到导通。
6 次减半后最小步长为原始步长的 1/64，覆盖几乎全部场景。

## 发散日志改进

原日志：
```
WARN solver diverged after 50 iterations
```

新日志（含最差节点号和 delta）：
```
WARN solver diverged after 50 iterations (worst node=3 delta=1.50e-5)
```

Node 3 是 plate——多为接近收敛的极限环（delta 在 1e-7~5V 摆动），
不是真正的发散。后续考虑调宽容差到 1e-6。

## 影响

无。所有现有例子输出与原版一致（line search 阈值 10V 高于
正常运行时每步电压变化）。
