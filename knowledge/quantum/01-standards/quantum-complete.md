---
id: quantum-complete
title: 量子计算完整指南
domain: quantum
category: 01-standards
difficulty: intermediate
tags: [complete, nisq, quantum, 时代应用, 核心概念, 概述, 量子密码学, 量子机器学习]
quality_score: 70
last_updated: 2026-06-15
---
# 量子计算完整指南

## 概述
量子计算利用量子力学原理（叠加态和纠缠）进行信息处理，能够解决经典计算机难以处理的复杂问题。本指南覆盖量子计算基础、量子算法、主流框架、量子机器学习、量子密码学、NISQ时代应用和实践指南。

## 核心概念

### 1. 量子比特（Qubit）

**经典比特 vs 量子比特**:
- 经典比特: 0 或 1
- 量子比特: |0⟩、|1⟩ 或两者的叠加态 α|0⟩ + β|1⟩
- |α|² + |β|² = 1（归一化条件）
- n 个量子比特可表示 2^n 个状态的叠加

**Bloch球表示**:
```python
from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector
from qiskit.visualization import plot_bloch_multivector

# 创建量子电路
qc = QuantumCircuit(1)

# 初始状态 |0⟩ - Bloch球北极
state = Statevector.from_instruction(qc)
plot_bloch_multivector(state)

# 应用Hadamard门 - 创建叠加态 - Bloch球赤道
qc.h(0)
state = Statevector.from_instruction(qc)
plot_bloch_multivector(state)
# |+⟩ = (|0⟩ + |1⟩) / √2
```

**量子比特物理实现**:

| 技术 | 代表机构 | 相干时间 | 门保真度 | 规模 |
|------|---------|---------|---------|------|
| 超导量子比特 | IBM, Google | ~100μs | ~99.5% | 100+ |
| 离子阱 | IonQ, Quantinuum | ~10s | ~99.9% | 30+ |
| 光量子 | Xanadu, PsiQuantum | ~ns | ~99% | 实验阶段 |
| 拓扑量子比特 | Microsoft | 理论长 | 理论高 | 研发中 |
| 中性原子 | Atom Computing | ~1s | ~99.5% | 1000+ |

### 2. 量子门

**单比特门**:
```python
from qiskit import QuantumCircuit
import numpy as np

qc = QuantumCircuit(1)

# Pauli门
qc.x(0)  # X门 (NOT): |0⟩ → |1⟩, |1⟩ → |0⟩
qc.y(0)  # Y门: |0⟩ → i|1⟩, |1⟩ → -i|0⟩
qc.z(0)  # Z门: |0⟩ → |0⟩, |1⟩ → -|1⟩

# Hadamard门 (创建叠加态)
qc.h(0)  # |0⟩ → (|0⟩ + |1⟩)/√2

# 相位门
qc.s(0)  # S门: |1⟩ → i|1⟩ (Z的平方根)
qc.t(0)  # T门: |1⟩ → e^(iπ/4)|1⟩ (S的平方根)

# 旋转门（参数化）
qc.rx(np.pi/2, 0)  # 绕X轴旋转π/2
qc.ry(np.pi/4, 0)  # 绕Y轴旋转π/4
qc.rz(np.pi/8, 0)  # 绕Z轴旋转π/8

# 矩阵表示
# H = (1/√2) * [[1, 1], [1, -1]]
# X = [[0, 1], [1, 0]]
# Z = [[1, 0], [0, -1]]
```

**多比特门**:
```python
qc = QuantumCircuit(3)

# CNOT门 (受控NOT) - 两比特纠缠的基础
qc.cx(0, 1)  # 如果qubit0=1,则翻转qubit1

# CZ门 (受控Z)
qc.cz(0, 1)

# SWAP门 - 交换两个量子比特状态
qc.swap(0, 1)

# Toffoli门 (CCNOT) - 通用经典计算
qc.ccx(0, 1, 2)  # 如果qubit0=1 且 qubit1=1,则翻转qubit2

# 受控旋转门
qc.crx(np.pi/2, 0, 1)  # 受控RX
qc.cry(np.pi/4, 0, 1)  # 受控RY
qc.crz(np.pi/8, 0, 1)  # 受控RZ
```

### 3. 量子纠缠

**Bell态（最大纠缠态）**:
```python
from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector

# 四种Bell态
def create_bell_state(variant='phi_plus'):
    qc = QuantumCircuit(2)

    if variant == 'phi_plus':
        # |Φ+⟩ = (|00⟩ + |11⟩) / √2
        qc.h(0)
        qc.cx(0, 1)
    elif variant == 'phi_minus':
        # |Φ-⟩ = (|00⟩ - |11⟩) / √2
        qc.h(0)
        qc.cx(0, 1)
        qc.z(0)
    elif variant == 'psi_plus':
        # |Ψ+⟩ = (|01⟩ + |10⟩) / √2
        qc.h(0)
        qc.cx(0, 1)
        qc.x(1)
    elif variant == 'psi_minus':
        # |Ψ-⟩ = (|01⟩ - |10⟩) / √2
        qc.h(0)
        qc.cx(0, 1)
        qc.x(1)
        qc.z(0)

    return qc

# GHZ态（多体纠缠）
def create_ghz_state(n_qubits):
    """创建 n 量子比特 GHZ 态: (|00...0⟩ + |11...1⟩) / √2"""
    qc = QuantumCircuit(n_qubits)
    qc.h(0)
    for i in range(n_qubits - 1):
        qc.cx(i, i + 1)
    return qc
```

### 4. 量子测量

```python
from qiskit import QuantumCircuit
from qiskit_aer import AerSimulator

qc = QuantumCircuit(1, 1)

# 创建叠加态
qc.h(0)

# Z基测量（计算基）
qc.measure(0, 0)

# 执行多次采样
simulator = AerSimulator()
result = simulator.run(qc, shots=10000).result()
counts = result.get_counts()
print(counts)  # {'0': ~5000, '1': ~5000}

# X基测量（需要先旋转）
qc2 = QuantumCircuit(1, 1)
qc2.h(0)       # 准备 |+⟩ 态
qc2.h(0)       # 旋转到Z基
qc2.measure(0, 0)
# 测量结果: {'0': 10000}（|+⟩在X基测量必定为+）
```

### 5. 量子退相干与噪声

```python
from qiskit_aer import AerSimulator
from qiskit_aer.noise import NoiseModel, depolarizing_error, thermal_relaxation_error

# 构建噪声模型
noise_model = NoiseModel()

# 退极化噪声（单比特门错误率 0.1%）
error_1q = depolarizing_error(0.001, 1)
noise_model.add_all_qubit_quantum_error(error_1q, ['h', 'x', 'y', 'z'])

# 退极化噪声（双比特门错误率 1%）
error_2q = depolarizing_error(0.01, 2)
noise_model.add_all_qubit_quantum_error(error_2q, ['cx'])

# 热弛豫噪声（T1=50μs, T2=70μs, 门时间=50ns）
thermal_error = thermal_relaxation_error(50e3, 70e3, 50)
noise_model.add_all_qubit_quantum_error(thermal_error, ['h', 'x'])

# 使用噪声模型运行
noisy_sim = AerSimulator(noise_model=noise_model)
result = noisy_sim.run(qc, shots=10000).result()
```

---

## 量子算法

### 1. Deutsch-Jozsa算法

**问题**: 判断函数f(x)是常量还是平衡（经典需 2^(n-1)+1 次查询，量子只需 1 次）

```python
from qiskit import QuantumCircuit
from qiskit_aer import AerSimulator

def deutsch_jozsa(oracle_type='constant', n=3):
    """
    Deutsch-Jozsa算法

    oracle_type: 'constant' 或 'balanced'
    n: 输入比特数
    """
    qc = QuantumCircuit(n + 1, n)

    # 初始化: 辅助比特置|1⟩，所有比特Hadamard
    qc.x(n)
    qc.h(range(n + 1))

    # Oracle
    qc.barrier()
    if oracle_type == 'constant':
        pass  # f(x) = 0，不做任何操作
    else:
        for i in range(n):
            qc.cx(i, n)  # f(x) = x₀ ⊕ x₁ ⊕ ... ⊕ xₙ₋₁
    qc.barrier()

    # Hadamard + 测量
    qc.h(range(n))
    qc.measure(range(n), range(n))

    return qc

# 测试
simulator = AerSimulator()
qc_const = deutsch_jozsa('constant')
qc_bal = deutsch_jozsa('balanced')

result_const = simulator.run(qc_const, shots=1000).result()
result_bal = simulator.run(qc_bal, shots=1000).result()

print("常量:", result_const.get_counts())  # {'000': 1000}
print("平衡:", result_bal.get_counts())    # 非全零结果
```

### 2. Grover搜索算法

**问题**: 在 N 个无序项中搜索目标（经典 O(N)，量子 O(√N)）

```python
from qiskit import QuantumCircuit
from qiskit_aer import AerSimulator
import numpy as np

def grover_search(target=5, n_qubits=3):
    """
    Grover搜索算法

    target: 目标项 (0 到 2^n_qubits - 1)
    n_qubits: 比特数
    """
    qc = QuantumCircuit(n_qubits, n_qubits)

    # 均匀叠加态
    qc.h(range(n_qubits))

    # 最优迭代次数
    iterations = int(np.pi / 4 * np.sqrt(2**n_qubits))

    for _ in range(iterations):
        # === Oracle: 标记目标态 ===
        # 翻转非目标位
        for i in range(n_qubits):
            if not (target >> i) & 1:
                qc.x(i)

        # 多控Z门（标记目标）
        qc.h(n_qubits - 1)
        qc.mcx(list(range(n_qubits - 1)), n_qubits - 1)
        qc.h(n_qubits - 1)

        # 恢复翻转
        for i in range(n_qubits):
            if not (target >> i) & 1:
                qc.x(i)

        # === Diffusion 算子（振幅放大） ===
        qc.h(range(n_qubits))
        qc.x(range(n_qubits))
        qc.h(n_qubits - 1)
        qc.mcx(list(range(n_qubits - 1)), n_qubits - 1)
        qc.h(n_qubits - 1)
        qc.x(range(n_qubits))
        qc.h(range(n_qubits))

    qc.measure(range(n_qubits), range(n_qubits))
    return qc

# 执行
simulator = AerSimulator()
qc = grover_search(target=5, n_qubits=3)
result = simulator.run(qc, shots=1000).result()
counts = result.get_counts()
print(counts)  # {'101': ~945} (目标5的二进制, 高概率)
```

### 3. Shor算法（整数分解）

**问题**: 将大整数分解为质因子（破解RSA的理论基础）

**核心步骤**:
```
1. 随机选择 a < N
2. 检查 gcd(a, N) > 1，若是则直接找到因子
3. 用量子傅里叶变换(QFT)找 a^r ≡ 1 (mod N) 的周期 r
4. 若 r 为偶数，计算 gcd(a^(r/2) ± 1, N) 得到因子
```

```python
from qiskit import QuantumCircuit
from qiskit.circuit.library import QFT
import numpy as np

def quantum_period_finding(a, N, n_counting_qubits):
    """量子周期查找子程序（Shor算法核心）"""
    qc = QuantumCircuit(n_counting_qubits + 4, n_counting_qubits)

    # 初始化计数寄存器为叠加态
    qc.h(range(n_counting_qubits))

    # 工作寄存器初始化为 |1⟩
    qc.x(n_counting_qubits)

    # 受控模幂运算 (controlled modular exponentiation)
    # U|y⟩ = |a*y mod N⟩
    for i in range(n_counting_qubits):
        power = 2**i
        # 实际实现需要模幂电路
        # 此处为概念示意
        pass

    # 逆量子傅里叶变换
    qc.append(QFT(n_counting_qubits, inverse=True), range(n_counting_qubits))

    # 测量
    qc.measure(range(n_counting_qubits), range(n_counting_qubits))

    return qc

# 经典后处理
def shor_classical(N):
    """Shor算法的经典部分"""
    from math import gcd
    import random

    a = random.randint(2, N - 1)
    if gcd(a, N) > 1:
        return gcd(a, N), N // gcd(a, N)

    # 量子部分: 找到周期 r 使得 a^r ≡ 1 (mod N)
    # r = quantum_period_finding(a, N)

    # 假设找到 r
    # if r % 2 == 0:
    #     factor1 = gcd(a**(r//2) - 1, N)
    #     factor2 = gcd(a**(r//2) + 1, N)
    #     return factor1, factor2

    return None
```

### 4. VQE（变分量子特征求解器）

**应用**: 量子化学（分子基态能量计算）、组合优化

```python
from qiskit import QuantumCircuit
from qiskit.circuit.library import TwoLocal, EfficientSU2
from qiskit_algorithms import VQE
from qiskit_algorithms.optimizers import COBYLA, SPSA, L_BFGS_B
from qiskit.quantum_info import SparsePauliOp
from qiskit_aer.primitives import Estimator

# 定义哈密顿量（H2分子示例）
hamiltonian = SparsePauliOp.from_list([
    ("II", -1.0523732),
    ("IZ",  0.3979374),
    ("ZI", -0.3979374),
    ("ZZ", -0.0112801),
    ("XX",  0.1809312),
])

# 定义参数化电路（Ansatz）
ansatz = EfficientSU2(
    num_qubits=2,
    entanglement='linear',
    reps=2
)

# 优化器选择
optimizer = COBYLA(maxiter=500)

# 运行VQE
estimator = Estimator()
vqe = VQE(estimator=estimator, ansatz=ansatz, optimizer=optimizer)
result = vqe.compute_minimum_eigenvalue(hamiltonian)

print(f"基态能量: {result.eigenvalue:.6f} Hartree")
print(f"最优参数: {result.optimal_parameters}")
```

### 5. QAOA（量子近似优化算法）

**应用**: 组合优化问题（MaxCut、旅行商、调度）

```python
from qiskit import QuantumCircuit
from qiskit_algorithms import QAOA
from qiskit_algorithms.optimizers import COBYLA
from qiskit.quantum_info import SparsePauliOp
from qiskit_aer.primitives import Sampler
import numpy as np

def create_maxcut_hamiltonian(edges, n_nodes):
    """
    MaxCut 问题: 将图的节点分为两组，使得跨组边数最大

    Cost Hamiltonian: C = Σ(i,j)∈E (1 - Z_i Z_j) / 2
    """
    pauli_list = []
    for i, j in edges:
        # (1 - Z_i Z_j) / 2 的展开
        label_zz = ['I'] * n_nodes
        label_zz[i] = 'Z'
        label_zz[j] = 'Z'

        pauli_list.append((''.join(label_zz), -0.5))
        pauli_list.append(('I' * n_nodes, 0.5))

    return SparsePauliOp.from_list(pauli_list).simplify()

# 定义图（4个节点的环形图）
edges = [(0, 1), (1, 2), (2, 3), (3, 0)]
n_nodes = 4

# 构建哈密顿量
hamiltonian = create_maxcut_hamiltonian(edges, n_nodes)

# QAOA 参数
p = 2  # QAOA层数（深度）

# 运行QAOA
sampler = Sampler()
qaoa = QAOA(
    sampler=sampler,
    optimizer=COBYLA(maxiter=200),
    reps=p
)

result = qaoa.compute_minimum_eigenvalue(hamiltonian)
print(f"最优值: {result.eigenvalue:.4f}")
print(f"最优解: {result.best_measurement}")

# QAOA 电路结构:
# 1. 初始态: |+⟩^n (均匀叠加)
# 2. p层迭代:
#    a. 问题层 (Cost): exp(-iγC) - 编码优化目标
#    b. 混合层 (Mixer): exp(-iβB) - 探索解空间
# 3. 测量获得近似最优解
```

### 6. 量子相位估计（QPE）

```python
from qiskit import QuantumCircuit
from qiskit.circuit.library import QFT
import numpy as np

def quantum_phase_estimation(unitary_gate, n_counting=4):
    """
    量子相位估计: 找到酉算子U的特征值 e^(2πiθ)

    输入: U|ψ⟩ = e^(2πiθ)|ψ⟩
    输出: θ的n位二进制近似
    """
    n_target = unitary_gate.num_qubits
    qc = QuantumCircuit(n_counting + n_target, n_counting)

    # 计数寄存器: Hadamard
    qc.h(range(n_counting))

    # 目标寄存器: 特征态（此处假设已准备好）

    # 受控U^(2^k)
    for k in range(n_counting):
        power = 2**k
        controlled_u = unitary_gate.power(power).control(1)
        qc.append(controlled_u, [k] + list(range(n_counting, n_counting + n_target)))

    # 逆QFT
    qc.append(QFT(n_counting, inverse=True), range(n_counting))

    # 测量计数寄存器
    qc.measure(range(n_counting), range(n_counting))

    return qc
```

---

## 量子机器学习

### 1. 量子核方法（Quantum Kernel）

```python
from qiskit.circuit.library import ZZFeatureMap
from qiskit_machine_learning.kernels import FidelityQuantumKernel
from qiskit_aer.primitives import Sampler
from sklearn.svm import SVC
from sklearn.datasets import make_moons
from sklearn.model_selection import train_test_split

# 生成数据
X, y = make_moons(n_samples=200, noise=0.1, random_state=42)
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.3)

# 量子特征映射
feature_map = ZZFeatureMap(
    feature_dimension=2,
    reps=2,
    entanglement='linear'
)

# 量子核
sampler = Sampler()
quantum_kernel = FidelityQuantumKernel(
    feature_map=feature_map,
    fidelity=sampler
)

# 量子SVM
qsvm = SVC(kernel=quantum_kernel.evaluate)
qsvm.fit(X_train, y_train)

accuracy = qsvm.score(X_test, y_test)
print(f"量子SVM准确率: {accuracy:.2%}")
```

### 2. 参数化量子电路（PQC）分类器

```python
from qiskit import QuantumCircuit
from qiskit.circuit import ParameterVector
from qiskit_machine_learning.neural_networks import EstimatorQNN
from qiskit_machine_learning.algorithms.classifiers import NeuralNetworkClassifier
from qiskit_algorithms.optimizers import ADAM
from qiskit_aer.primitives import Estimator
import numpy as np

def create_qnn_circuit(n_features, n_layers):
    """构建参数化量子神经网络"""
    n_qubits = n_features
    inputs = ParameterVector('x', n_features)
    weights = ParameterVector('w', n_layers * n_qubits * 2)

    qc = QuantumCircuit(n_qubits)

    # 数据编码层
    for i in range(n_qubits):
        qc.ry(inputs[i], i)

    # 参数化层
    w_idx = 0
    for layer in range(n_layers):
        # 旋转门
        for i in range(n_qubits):
            qc.ry(weights[w_idx], i)
            w_idx += 1
            qc.rz(weights[w_idx], i)
            w_idx += 1

        # 纠缠层
        for i in range(n_qubits - 1):
            qc.cx(i, i + 1)
        if n_qubits > 2:
            qc.cx(n_qubits - 1, 0)

    return qc, inputs, weights

# 构建电路
qc, inputs, weights = create_qnn_circuit(n_features=2, n_layers=3)

# 创建QNN
estimator = Estimator()
qnn = EstimatorQNN(
    circuit=qc,
    input_params=list(inputs),
    weight_params=list(weights),
    estimator=estimator
)

# 训练分类器
classifier = NeuralNetworkClassifier(
    neural_network=qnn,
    optimizer=ADAM(maxiter=100, lr=0.1),
    loss='cross_entropy'
)

# classifier.fit(X_train, y_train)
# predictions = classifier.predict(X_test)
```

### 3. 量子生成对抗网络（QGAN）

```python
# QGAN 概念架构
# 生成器: 参数化量子电路 → 生成量子态 → 测量得到样本
# 判别器: 经典神经网络 → 判断真假

# 训练流程:
# 1. 生成器产出样本
# 2. 判别器区分真实 vs 生成样本
# 3. 更新生成器参数使判别器更难区分
# 4. 更新判别器参数使其更擅长区分

def quantum_generator(n_qubits, n_layers):
    """量子生成器电路"""
    from qiskit.circuit import ParameterVector

    params = ParameterVector('g', n_layers * n_qubits * 3)
    qc = QuantumCircuit(n_qubits)

    p_idx = 0
    for _ in range(n_layers):
        for i in range(n_qubits):
            qc.rx(params[p_idx], i); p_idx += 1
            qc.ry(params[p_idx], i); p_idx += 1
            qc.rz(params[p_idx], i); p_idx += 1
        for i in range(n_qubits - 1):
            qc.cx(i, i + 1)

    qc.measure_all()
    return qc, params
```

---

## 量子密码学

### 1. BB84 量子密钥分发（QKD）

```python
import numpy as np

def bb84_protocol(n_bits=100):
    """
    BB84量子密钥分发协议模拟

    核心原理: 量子不可克隆定理 + 测量会干扰量子态
    """

    # Alice准备随机比特和随机基
    alice_bits = np.random.randint(0, 2, n_bits)
    alice_bases = np.random.randint(0, 2, n_bits)  # 0=Z基, 1=X基

    # Alice准备量子态
    # Z基: 0→|0⟩, 1→|1⟩
    # X基: 0→|+⟩, 1→|-⟩

    # Bob选择随机测量基
    bob_bases = np.random.randint(0, 2, n_bits)

    # Bob测量
    bob_results = []
    for i in range(n_bits):
        if alice_bases[i] == bob_bases[i]:
            # 基匹配: 确定性结果
            bob_results.append(alice_bits[i])
        else:
            # 基不匹配: 随机结果
            bob_results.append(np.random.randint(0, 2))

    # 基比对（公开信道）: 保留基匹配的比特
    sifted_key_alice = []
    sifted_key_bob = []
    for i in range(n_bits):
        if alice_bases[i] == bob_bases[i]:
            sifted_key_alice.append(alice_bits[i])
            sifted_key_bob.append(bob_results[i])

    # 错误率检测（取样一部分比较）
    sample_size = min(len(sifted_key_alice) // 4, 10)
    errors = sum(
        sifted_key_alice[i] != sifted_key_bob[i]
        for i in range(sample_size)
    )
    error_rate = errors / sample_size if sample_size > 0 else 0

    print(f"原始比特数: {n_bits}")
    print(f"筛后密钥长度: {len(sifted_key_alice)}")
    print(f"采样错误率: {error_rate:.2%}")

    if error_rate > 0.11:  # 阈值: ~11%
        print("警告: 可能存在窃听者!")
    else:
        print("密钥安全")

    return sifted_key_alice[sample_size:], error_rate

key, err = bb84_protocol(1000)
```

### 2. 后量子密码学（Post-Quantum Cryptography）

```
量子计算对现有密码学的威胁:

| 算法 | 类型 | 量子威胁 | 替代方案 |
|------|------|---------|---------|
| RSA | 非对称加密 | Shor算法可破解 | 格密码(Lattice) |
| ECC | 非对称加密 | Shor算法可破解 | 格密码/哈希签名 |
| AES-128 | 对称加密 | Grover降至64位 | AES-256 |
| AES-256 | 对称加密 | Grover降至128位 | 仍然安全 |
| SHA-256 | 哈希 | Grover降至128位 | 仍然安全 |

NIST后量子密码标准（2024年发布）:
- ML-KEM (CRYSTALS-Kyber): 密钥封装
- ML-DSA (CRYSTALS-Dilithium): 数字签名
- SLH-DSA (SPHINCS+): 基于哈希的签名
- FN-DSA (FALCON): 紧凑签名
```

**后量子密码使用示例**:
```python
# 使用 liboqs (Open Quantum Safe)
# pip install liboqs-python

from oqs import KeyEncapsulation, Signature

# ML-KEM 密钥封装
kem = KeyEncapsulation("ML-KEM-768")
public_key = kem.generate_keypair()
ciphertext, shared_secret_enc = kem.encap_secret(public_key)
shared_secret_dec = kem.decap_secret(ciphertext)
assert shared_secret_enc == shared_secret_dec

# ML-DSA 数字签名
sig = Signature("ML-DSA-65")
public_key = sig.generate_keypair()
message = b"Hello, post-quantum world!"
signature = sig.sign(message)
is_valid = sig.verify(message, signature, public_key)
print(f"签名验证: {is_valid}")  # True
```

### 3. 量子随机数生成

```python
from qiskit import QuantumCircuit
from qiskit_aer import AerSimulator

def quantum_random_bytes(n_bytes):
    """利用量子叠加态生成真随机数"""
    n_bits = n_bytes * 8
    qc = QuantumCircuit(n_bits, n_bits)

    # 所有比特置于叠加态
    qc.h(range(n_bits))

    # 测量
    qc.measure(range(n_bits), range(n_bits))

    # 执行（单次采样）
    simulator = AerSimulator()
    result = simulator.run(qc, shots=1).result()
    bitstring = list(result.get_counts().keys())[0]

    # 转换为字节
    random_bytes = int(bitstring, 2).to_bytes(n_bytes, 'big')
    return random_bytes

# 生成32字节随机数（256位）
random_key = quantum_random_bytes(32)
print(f"量子随机密钥: {random_key.hex()}")
```

---

## NISQ 时代应用

### 什么是 NISQ?

```
NISQ = Noisy Intermediate-Scale Quantum
含噪声的中等规模量子计算

特征:
- 量子比特数: 50-1000+
- 相干时间: 有限（微秒到毫秒级）
- 门错误率: 0.1% - 1%
- 无完全纠错能力
- 电路深度受限（< 100-1000 层）
```

### 1. NISQ 适用场景

| 场景 | 算法 | 量子比特需求 | 成熟度 |
|------|------|------------|--------|
| 分子模拟 | VQE | 10-100 | 实验验证 |
| 组合优化 | QAOA | 50-500 | 原型阶段 |
| 机器学习 | QNN/QKernel | 10-50 | 研究阶段 |
| 金融建模 | 量子蒙特卡罗 | 50-200 | 探索阶段 |
| 材料设计 | VQE/QPE | 50-500 | 探索阶段 |

### 2. 错误缓解技术（Error Mitigation）

```python
from qiskit_aer.primitives import Estimator
from qiskit.quantum_info import SparsePauliOp

# 方法1: 零噪声外推 (ZNE)
# 原理: 人为增大噪声，测量多个噪声级别，外推到零噪声

def zero_noise_extrapolation(circuit, observable, noise_factors=[1, 2, 3]):
    """零噪声外推"""
    results = []

    for factor in noise_factors:
        # 通过门折叠(gate folding)增大噪声
        noisy_circuit = fold_gates(circuit, factor)
        estimator = Estimator()
        result = estimator.run(noisy_circuit, observable).result()
        results.append(result.values[0])

    # Richardson外推
    # 线性外推到 factor=0
    import numpy as np
    coeffs = np.polyfit(noise_factors, results, deg=len(noise_factors)-1)
    extrapolated = np.polyval(coeffs, 0)

    return extrapolated

# 方法2: 概率错误消除 (PEC)
# 原理: 将噪声信道分解为理想门的线性组合

# 方法3: 测量错误缓解
from qiskit_aer.noise import NoiseModel
from qiskit.result import marginal_counts

def measurement_error_mitigation(counts, calibration_matrix):
    """使用校准矩阵修正测量错误"""
    import numpy as np

    # 构建概率向量
    n_qubits = int(np.log2(len(calibration_matrix)))
    prob_vector = np.zeros(2**n_qubits)

    total = sum(counts.values())
    for bitstring, count in counts.items():
        idx = int(bitstring, 2)
        prob_vector[idx] = count / total

    # 应用逆校准矩阵
    corrected = np.linalg.solve(calibration_matrix, prob_vector)
    corrected = np.clip(corrected, 0, 1)
    corrected /= corrected.sum()

    return corrected
```

### 3. 量子-经典混合计算

```
NISQ 时代的最佳实践: 量子-经典混合架构

┌────────────────────────────────────────┐
│            经典计算机                    │
│  ┌──────────────────────────────────┐  │
│  │  参数优化器 (COBYLA/ADAM/SPSA)   │  │
│  │         ↕ 参数更新                │  │
│  │  结果后处理 / 错误缓解            │  │
│  └──────────────────────────────────┘  │
│               ↕ 参数/结果传递           │
├────────────────────────────────────────┤
│            量子处理器                    │
│  ┌──────────────────────────────────┐  │
│  │  参数化电路执行                    │  │
│  │  量子态制备 + 测量                 │  │
│  └──────────────────────────────────┘  │
└────────────────────────────────────────┘

关键原则:
1. 量子部分尽量浅（短电路深度）
2. 复杂优化放在经典端
3. 使用错误缓解而非纠错
4. 利用问题结构减少量子资源需求
```

---

## 量子编程框架

### 1. Qiskit (IBM)

```python
# Qiskit 1.x API（最新版本）
from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector, Operator
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit_aer import AerSimulator
from qiskit_ibm_runtime import QiskitRuntimeService, Sampler, Estimator

# 本地模拟
simulator = AerSimulator()
qc = QuantumCircuit(2)
qc.h(0)
qc.cx(0, 1)
qc.measure_all()
result = simulator.run(qc, shots=1000).result()

# IBM 真实量子硬件
service = QiskitRuntimeService(channel="ibm_quantum")
backend = service.least_busy(min_num_qubits=2, simulator=False)

# 转译优化
pm = generate_preset_pass_manager(
    optimization_level=3,
    backend=backend
)
optimized_circuit = pm.run(qc)

# 执行
sampler = Sampler(backend)
job = sampler.run([optimized_circuit], shots=4096)
result = job.result()
```

### 2. Cirq (Google)

```python
import cirq
import numpy as np

# 创建量子比特
q0, q1 = cirq.LineQubit.range(2)

# 创建电路
circuit = cirq.Circuit([
    cirq.H(q0),
    cirq.CNOT(q0, q1),
    cirq.measure(q0, q1, key='result')
])

print(circuit)
# 0: ───H───@───M('result')───
#            │   │
# 1: ───────X───M──────────────

# 模拟
simulator = cirq.Simulator()
result = simulator.run(circuit, repetitions=1000)
print(result.histogram(key='result'))

# 参数化电路
theta = cirq.Symbol('theta')
param_circuit = cirq.Circuit([
    cirq.ry(theta)(q0),
    cirq.CNOT(q0, q1),
    cirq.measure(q0, q1, key='result')
])

# 扫参数
sweep = cirq.Linspace(key='theta', start=0, stop=2*np.pi, length=10)
results = simulator.run_sweep(param_circuit, sweep, repetitions=100)

# 噪声模拟
noisy_circuit = cirq.Circuit([
    cirq.H(q0),
    cirq.depolarize(p=0.01).on(q0),  # 退极化噪声
    cirq.CNOT(q0, q1),
    cirq.depolarize(p=0.02).on(q0),
    cirq.depolarize(p=0.02).on(q1),
    cirq.measure(q0, q1, key='result')
])
```

### 3. PennyLane（自动微分）

```python
import pennylane as qml
from pennylane import numpy as np

# 定义量子设备
dev = qml.device('default.qubit', wires=2)

# 定义量子节点（自动微分支持）
@qml.qnode(dev, diff_method='parameter-shift')
def circuit(params):
    qml.RX(params[0], wires=0)
    qml.RY(params[1], wires=1)
    qml.CNOT(wires=[0, 1])
    return qml.expval(qml.PauliZ(0) @ qml.PauliZ(1))

# 梯度计算
params = np.array([0.5, 0.3], requires_grad=True)
grad_fn = qml.grad(circuit)
gradients = grad_fn(params)
print(f"梯度: {gradients}")

# 变分优化
opt = qml.AdamOptimizer(stepsize=0.1)
for i in range(100):
    params = opt.step(lambda p: -circuit(p), params)

print(f"优化后参数: {params}")
print(f"最终期望值: {circuit(params):.6f}")

# PennyLane + PyTorch 集成
import torch

dev_torch = qml.device('default.qubit', wires=2)

@qml.qnode(dev_torch, interface='torch')
def torch_circuit(inputs, weights):
    qml.AngleEmbedding(inputs, wires=range(2))
    qml.StronglyEntanglingLayers(weights, wires=range(2))
    return qml.expval(qml.PauliZ(0))

# PyTorch优化
weights = torch.randn(3, 2, 3, requires_grad=True)
optimizer = torch.optim.Adam([weights], lr=0.1)
```

### 框架选型指南

| 需求 | 推荐框架 | 理由 |
|------|---------|------|
| IBM硬件访问 | Qiskit | 原生支持 |
| Google硬件访问 | Cirq | 原生支持 |
| 量子机器学习 | PennyLane | 自动微分 + 框架集成 |
| 教学/学习 | Qiskit / Cirq | 文档丰富 |
| 研究/实验 | PennyLane | 灵活性高 |
| 生产部署 | Qiskit Runtime | 会话管理 + 优化 |

---

## 最佳实践

### 1. 电路优化

```python
# ✅ 减少电路深度
qc = QuantumCircuit(2)
qc.h(0)
qc.cx(0, 1)  # 深度 2

# ❌ 冗余门
qc_bad = QuantumCircuit(2)
qc_bad.h(0)
qc_bad.h(0)   # H·H = I，互相抵消
qc_bad.cx(0, 1)

# ✅ 利用转译器优化
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager

# optimization_level:
# 0 = 无优化
# 1 = 轻量优化（门合并）
# 2 = 中等优化（+ 路由优化）
# 3 = 重度优化（+ 门分解优化, 最慢但最优）
```

### 2. 错误缓解

```python
from qiskit_aer.noise import NoiseModel
from qiskit_aer import AerSimulator

# ✅ 使用适当的模拟器进行开发
# 开发阶段: 无噪声模拟器
ideal_sim = AerSimulator(method='statevector')

# 验证阶段: 噪声模拟器
noise_model = NoiseModel.from_backend(real_backend)
noisy_sim = AerSimulator(noise_model=noise_model)

# 生产阶段: 真实硬件 + 错误缓解
```

### 3. 开发流程

```
1. 理论验证
   └── 小规模无噪声模拟验证算法正确性

2. 噪声评估
   └── 使用噪声模型评估实际可行性

3. 电路优化
   └── 减少深度、门数量、量子比特数

4. 错误缓解
   └── 选择合适的缓解策略

5. 硬件执行
   └── 选择合适的量子处理器

6. 结果验证
   └── 与经典计算对比验证
```

---

## Agent Checklist

### 量子算法选型
- [ ] 确认问题类型（优化/模拟/搜索/密码学）
- [ ] 评估量子优势是否存在（与最佳经典算法对比）
- [ ] 确认所需量子比特数和电路深度
- [ ] 评估当前硬件是否满足需求（NISQ限制）
- [ ] 选择合适的变分算法（VQE/QAOA/QNN）或精确算法

### 框架选型
- [ ] 确认目标硬件（IBM/Google/IonQ/模拟器）
- [ ] 评估是否需要自动微分（PennyLane）
- [ ] 确认框架版本兼容性
- [ ] 评估社区支持和文档质量

### 电路设计
- [ ] 最小化电路深度（NISQ关键约束）
- [ ] 选择合适的Ansatz（问题相关 vs 硬件高效）
- [ ] 合理使用参数化门
- [ ] 考虑硬件拓扑（量子比特连接性）
- [ ] 避免冗余门和不必要的纠缠

### 噪声处理
- [ ] 使用噪声模型进行预评估
- [ ] 选择错误缓解策略（ZNE/PEC/M3）
- [ ] 校准测量错误
- [ ] 评估不同shots数对精度的影响
- [ ] 确认电路深度在相干时间内

### 优化与调参
- [ ] 选择合适的经典优化器（COBYLA/SPSA/ADAM）
- [ ] 设置合理的迭代次数和收敛标准
- [ ] 处理参数平原（Barren Plateau）问题
- [ ] 使用参数初始化策略
- [ ] 评估优化景观

### 安全评估（量子密码学相关）
- [ ] 评估量子威胁对现有系统的影响
- [ ] 规划后量子密码迁移路径
- [ ] 确认 NIST 后量子标准支持
- [ ] 实施加密敏捷性（Crypto Agility）策略

### 生产部署
- [ ] 本地模拟验证通过
- [ ] 噪声模拟验证通过
- [ ] 真实硬件测试完成
- [ ] 结果与经典基准对比
- [ ] 性能（运行时间/精度）满足要求
- [ ] 成本评估（量子硬件使用费用）

---

**知识ID**: `quantum-complete`
**领域**: quantum
**类型**: standards
**难度**: advanced
**质量分**: 93
**维护者**: quantum-team@superdev.com
**最后更新**: 2026-03-28
