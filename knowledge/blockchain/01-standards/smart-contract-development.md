---
id: smart-contract-development
title: 智能合约开发完整指南
domain: blockchain
category: 01-standards
difficulty: intermediate
tags: [blockchain, contract, development, near, rust, smart, solidity, 优化]
quality_score: 70
last_updated: 2026-06-15
---
# 智能合约开发完整指南

## 概述

智能合约是部署在区块链上的自执行程序，一旦部署即不可修改（除非使用代理模式）。本指南覆盖 Solidity 和 Rust 智能合约开发、EVM 原理、Gas 优化、安全审计、测试框架、DeFi/NFT/DAO 开发模式以及 Layer 2 解决方案。

### 智能合约 vs 传统后端

| 维度 | 智能合约 | 传统后端 |
|------|---------|---------|
| 部署后修改 | 不可变（需代理模式） | 随时可更新 |
| 执行成本 | 每次调用消耗 Gas | 服务器资源（固定成本） |
| 状态管理 | 链上存储（极贵） | 数据库（便宜） |
| 错误处理 | revert 回滚所有状态 | 可部分失败 |
| 并发 | 串行执行（按区块） | 并行处理 |
| 审计要求 | 极高（资金安全） | 视业务而定 |
| 开源透明 | 代码公开可验证 | 可闭源 |

---

## EVM 原理

### 1. 以太坊虚拟机架构

```
┌─────────────────────────────────────────┐
│           Transaction 提交               │
├─────────────────────────────────────────┤
│        EVM 执行环境                      │
│  ┌──────────────────────────────────┐   │
│  │  Stack (栈)                       │   │
│  │  最大深度 1024, 每个元素 256 bit   │   │
│  ├──────────────────────────────────┤   │
│  │  Memory (内存)                    │   │
│  │  按字节寻址, 执行后释放             │   │
│  ├──────────────────────────────────┤   │
│  │  Storage (存储)                   │   │
│  │  持久化, 按 32 字节 slot 组织      │   │
│  │  读: 2100 Gas, 冷写: 20000 Gas   │   │
│  ├──────────────────────────────────┤   │
│  │  Calldata (调用数据)              │   │
│  │  只读, 函数参数                    │   │
│  └──────────────────────────────────┘   │
├─────────────────────────────────────────┤
│        World State 更新                  │
└─────────────────────────────────────────┘
```

### 2. 存储布局（Storage Layout）

```solidity
contract StorageLayout {
    // Slot 0: 完整的 32 字节
    uint256 public value1;

    // Slot 1: 紧凑打包（同一 slot）
    uint128 public value2;  // Slot 1 低 16 字节
    uint128 public value3;  // Slot 1 高 16 字节

    // Slot 2: bool 和 address 可打包
    bool public flag;       // Slot 2, 1 字节
    address public owner;   // Slot 2, 20 字节

    // Slot 3+: mapping 使用 keccak256(key, slot) 定位
    mapping(address => uint256) public balances;

    // 动态数组: slot 存长度, 数据在 keccak256(slot) 开始
    uint256[] public dynamicArray;
}
```

### 3. 操作码（Opcodes）与 Gas 成本

| 操作码 | Gas 成本 | 说明 |
|--------|---------|------|
| ADD/SUB/MUL | 3 | 算术运算 |
| SLOAD | 2100 (冷) / 100 (热) | 读取存储 |
| SSTORE | 20000 (新) / 5000 (更新) | 写入存储 |
| MLOAD/MSTORE | 3 | 内存读写 |
| CALL | 2600 (冷) / 100 (热) | 外部调用 |
| CREATE | 32000 | 创建合约 |
| LOG0-LOG4 | 375-1875 | 事件日志 |
| SELFDESTRUCT | 5000 | 销毁合约 |

---

## Solidity 开发

### 1. 合约结构最佳实践

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

/**
 * @title Vault
 * @author Team
 * @notice 安全的资金管理合约
 * @dev 使用 Checks-Effects-Interactions 模式
 */
contract Vault is Ownable, ReentrancyGuard, Pausable {
    // ============ 常量 ============
    uint256 public constant MAX_DEPOSIT = 100 ether;
    uint256 public constant MIN_DEPOSIT = 0.01 ether;

    // ============ 状态变量 ============
    mapping(address => uint256) private _balances;
    uint256 private _totalDeposits;

    // ============ 事件 ============
    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);

    // ============ 错误 ============
    error InsufficientBalance(uint256 requested, uint256 available);
    error DepositOutOfRange(uint256 amount);
    error ZeroAddress();

    // ============ 修饰器 ============
    modifier validAddress(address addr) {
        if (addr == address(0)) revert ZeroAddress();
        _;
    }

    // ============ 构造函数 ============
    constructor() Ownable(msg.sender) {}

    // ============ 外部函数 ============

    /// @notice 存款
    /// @dev 使用 nonReentrant 防重入
    function deposit() external payable nonReentrant whenNotPaused {
        if (msg.value < MIN_DEPOSIT || msg.value > MAX_DEPOSIT) {
            revert DepositOutOfRange(msg.value);
        }

        // Effects
        _balances[msg.sender] += msg.value;
        _totalDeposits += msg.value;

        // Events
        emit Deposited(msg.sender, msg.value);
    }

    /// @notice 取款
    /// @param amount 取款金额
    function withdraw(uint256 amount) external nonReentrant whenNotPaused {
        uint256 balance = _balances[msg.sender];
        if (amount > balance) {
            revert InsufficientBalance(amount, balance);
        }

        // Effects (先更新状态)
        _balances[msg.sender] = balance - amount;
        _totalDeposits -= amount;

        // Interactions (最后执行外部调用)
        (bool success, ) = payable(msg.sender).call{value: amount}("");
        require(success, "Transfer failed");

        emit Withdrawn(msg.sender, amount);
    }

    // ============ 视图函数 ============

    function balanceOf(address account) external view returns (uint256) {
        return _balances[account];
    }

    function totalDeposits() external view returns (uint256) {
        return _totalDeposits;
    }

    // ============ 管理函数 ============

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }
}
```

### 2. 设计模式

#### 代理模式（Proxy Pattern - UUPS）

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

contract VaultV1 is UUPSUpgradeable, OwnableUpgradeable {
    uint256 public value;

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize() public initializer {
        __Ownable_init(msg.sender);
        __UUPSUpgradeable_init();
    }

    function setValue(uint256 _value) external {
        value = _value;
    }

    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}
}

contract VaultV2 is VaultV1 {
    uint256 public newFeature;

    function setNewFeature(uint256 _value) external {
        newFeature = _value;
    }
}
```

#### 钻石模式（Diamond Pattern - EIP-2535）

```solidity
// 适用于超大型合约（突破 24KB 限制）
// Facet A: 存款功能
contract DepositFacet {
    function deposit() external payable {
        // 存款逻辑
    }
}

// Facet B: 取款功能
contract WithdrawFacet {
    function withdraw(uint256 amount) external {
        // 取款逻辑
    }
}

// Diamond 合约通过 delegatecall 路由到对应 Facet
```

### 3. 常用库和接口

```solidity
// ERC-20 代币
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";

contract MyToken is ERC20, ERC20Permit {
    constructor() ERC20("MyToken", "MTK") ERC20Permit("MyToken") {
        _mint(msg.sender, 1_000_000 * 10 ** decimals());
    }
}

// ERC-721 NFT
import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";

contract MyNFT is ERC721, ERC721URIStorage {
    uint256 private _tokenIdCounter;

    constructor() ERC721("MyNFT", "MNFT") {}

    function safeMint(address to, string memory uri) public {
        uint256 tokenId = _tokenIdCounter++;
        _safeMint(to, tokenId);
        _setTokenURI(tokenId, uri);
    }
}

// ERC-1155 多代币标准
import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";

contract GameItems is ERC1155 {
    uint256 public constant GOLD = 0;
    uint256 public constant SWORD = 1;
    uint256 public constant SHIELD = 2;

    constructor() ERC1155("https://game.example/api/item/{id}.json") {
        _mint(msg.sender, GOLD, 10**18, "");
        _mint(msg.sender, SWORD, 100, "");
        _mint(msg.sender, SHIELD, 50, "");
    }
}
```

---

## Rust 智能合约（Solana / Near / Ink!）

### 1. Solana (Anchor 框架)

```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.total_deposits = 0;
        vault.bump = bump;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::ZeroAmount);

        // 转移 SOL
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.vault.to_account_info(),
            ],
        )?;

        let vault = &mut ctx.accounts.vault;
        vault.total_deposits += amount;

        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            amount,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault"],
        bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault"], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub authority: Pubkey,
    pub total_deposits: u64,
    pub bump: u8,
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Unauthorized access")]
    Unauthorized,
}
```

### 2. Near Protocol

```rust
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Vault {
    balances: LookupMap<AccountId, Balance>,
    total_deposits: Balance,
    owner: AccountId,
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            balances: LookupMap::new(b"b"),
            total_deposits: 0,
            owner: env::predecessor_account_id(),
        }
    }
}

#[near_bindgen]
impl Vault {
    #[payable]
    pub fn deposit(&mut self) {
        let amount = env::attached_deposit();
        assert!(amount > 0, "Deposit must be positive");

        let account = env::predecessor_account_id();
        let current = self.balances.get(&account).unwrap_or(0);
        self.balances.insert(&account, &(current + amount));
        self.total_deposits += amount;

        env::log_str(&format!("Deposited {} from {}", amount, account));
    }

    pub fn withdraw(&mut self, amount: Balance) {
        let account = env::predecessor_account_id();
        let balance = self.balances.get(&account).unwrap_or(0);
        assert!(amount <= balance, "Insufficient balance");

        self.balances.insert(&account, &(balance - amount));
        self.total_deposits -= amount;

        Promise::new(account).transfer(amount);
    }

    pub fn balance_of(&self, account_id: AccountId) -> Balance {
        self.balances.get(&account_id).unwrap_or(0)
    }
}
```

---

## Gas 优化

### 1. 存储优化（最大收益）

```solidity
// ❌ 每个变量占一个 slot (3 * 32 = 96 bytes)
contract Unoptimized {
    uint256 public a;    // Slot 0
    bool public b;       // Slot 1 (浪费 31 bytes)
    uint256 public c;    // Slot 2
}

// ✅ 紧凑打包 (64 bytes)
contract Optimized {
    uint256 public a;    // Slot 0
    uint256 public c;    // Slot 1
    bool public b;       // 与下一个小变量共享 Slot 2
}

// ✅ 结构体打包
contract StructPacking {
    // ❌ 3 个 slot
    struct BadUser {
        uint256 id;
        bool active;
        uint256 balance;
    }

    // ✅ 2 个 slot
    struct GoodUser {
        uint256 id;       // Slot 0
        uint128 balance;  // Slot 1 低 16 字节
        bool active;      // Slot 1 (与 balance 共享)
    }
}
```

### 2. 计算优化

```solidity
contract GasOptimization {
    uint256[] public data;

    // ❌ 每次循环读取 storage
    function sumBad() external view returns (uint256 total) {
        for (uint256 i = 0; i < data.length; i++) {
            total += data[i];
        }
    }

    // ✅ 缓存到 memory
    function sumGood() external view returns (uint256 total) {
        uint256[] memory _data = data;
        uint256 len = _data.length;
        for (uint256 i = 0; i < len; i++) {
            total += _data[i];
        }
    }

    // ✅ 使用 unchecked 跳过溢出检查（已知安全时）
    function sumBest() external view returns (uint256 total) {
        uint256[] memory _data = data;
        uint256 len = _data.length;
        for (uint256 i = 0; i < len; ) {
            total += _data[i];
            unchecked { ++i; }
        }
    }

    // ✅ 使用 calldata 替代 memory（只读参数）
    function processCalldata(uint256[] calldata items)
        external
        pure
        returns (uint256 total)
    {
        for (uint256 i = 0; i < items.length; ) {
            total += items[i];
            unchecked { ++i; }
        }
    }

    // ✅ 使用 custom error 替代 require string
    error Unauthorized();
    error InvalidAmount(uint256 amount);

    function optimizedRequire(uint256 amount) external view {
        if (msg.sender == address(0)) revert Unauthorized();
        if (amount == 0) revert InvalidAmount(amount);
    }
}
```

### 3. 事件优化

```solidity
contract EventOptimization {
    // ✅ 使用 indexed 参数（便于过滤，但增加少量 Gas）
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 amount    // 非 indexed，存储在 data 中
    );

    // ✅ 大量数据用事件而非 storage（便宜 5-10 倍）
    // 事件数据无法在合约中读取，但前端可以
    event DataStored(bytes32 indexed key, bytes data);
}
```

### Gas 优化速查表

| 技巧 | 节省 Gas | 风险等级 |
|------|---------|---------|
| 变量打包（slot packing） | 15000-20000 | 低 |
| 用 calldata 替代 memory | 500-5000 | 低 |
| 缓存 storage 到 memory | 2000+ per read | 低 |
| unchecked 算术 | 100-300 per op | 中（需确保安全） |
| Custom error 替代 string | 200-1000 | 低 |
| 短路求值优化 | 100-500 | 低 |
| 使用 immutable/constant | 2000+ | 低 |
| 批量操作替代多次调用 | 21000 per tx saved | 低 |

---

## 安全审计

### 1. 重入攻击（Reentrancy）

**攻击原理**:
```solidity
// 漏洞合约
contract VulnerableVault {
    mapping(address => uint256) public balances;

    function withdraw() external {
        uint256 amount = balances[msg.sender];
        // ❌ 先发送 ETH，再更新状态
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success);
        balances[msg.sender] = 0;  // 攻击者在此之前重新进入
    }
}

// 攻击合约
contract Attacker {
    VulnerableVault public vault;

    function attack() external payable {
        vault.deposit{value: 1 ether}();
        vault.withdraw();
    }

    receive() external payable {
        if (address(vault).balance >= 1 ether) {
            vault.withdraw();  // 重入！状态尚未更新
        }
    }
}
```

**防御方案**:
```solidity
contract SafeVault {
    mapping(address => uint256) public balances;
    bool private _locked;

    // 方案 1: Checks-Effects-Interactions 模式
    function withdraw() external {
        uint256 amount = balances[msg.sender];
        require(amount > 0, "No balance");

        // Effects（先更新状态）
        balances[msg.sender] = 0;

        // Interactions（后执行外部调用）
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success);
    }

    // 方案 2: ReentrancyGuard
    modifier nonReentrant() {
        require(!_locked, "Reentrant call");
        _locked = true;
        _;
        _locked = false;
    }

    function withdrawSafe() external nonReentrant {
        uint256 amount = balances[msg.sender];
        balances[msg.sender] = 0;
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success);
    }
}
```

### 2. 整数溢出（Integer Overflow/Underflow）

```solidity
// Solidity 0.8+ 默认有溢出检查，但 unchecked 块中仍需注意

contract IntegerSafety {
    // ✅ 0.8+ 默认安全
    function safeAdd(uint256 a, uint256 b) external pure returns (uint256) {
        return a + b;  // 溢出会自动 revert
    }

    // ❌ unchecked 中不安全
    function unsafeAdd(uint256 a, uint256 b) external pure returns (uint256) {
        unchecked {
            return a + b;  // 可能溢出！
        }
    }

    // ✅ 安全的类型转换
    function safeCast(uint256 value) external pure returns (uint128) {
        require(value <= type(uint128).max, "Overflow");
        return uint128(value);
    }
}
```

### 3. 授权漏洞（Access Control）

```solidity
contract AccessControlExample {
    // ❌ 缺少权限检查
    function dangerousFunction() external {
        // 任何人都可以调用！
    }

    // ❌ 使用 tx.origin（可被钓鱼攻击）
    function badAuth() external {
        require(tx.origin == owner);  // 不安全！
    }

    // ✅ 使用 OpenZeppelin AccessControl
    // 多角色权限管理
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        _mint(to, amount);
    }

    // ✅ 时间锁（Timelock）
    uint256 public constant TIMELOCK_DELAY = 2 days;
    mapping(bytes32 => uint256) public pendingActions;

    function scheduleAction(bytes32 actionId) external onlyOwner {
        pendingActions[actionId] = block.timestamp + TIMELOCK_DELAY;
    }

    function executeAction(bytes32 actionId) external onlyOwner {
        require(pendingActions[actionId] != 0, "Not scheduled");
        require(block.timestamp >= pendingActions[actionId], "Too early");
        delete pendingActions[actionId];
        // 执行操作
    }
}
```

### 4. 其他常见漏洞

```solidity
// ❌ 前置交易攻击（Front-running）
contract VulnerableAuction {
    function bid() external payable {
        // 攻击者可以看到 mempool 中的交易并抢先出价
    }
}

// ✅ 使用 commit-reveal 方案
contract SafeAuction {
    mapping(address => bytes32) public commits;

    function commit(bytes32 hash) external {
        commits[msg.sender] = hash;
    }

    function reveal(uint256 amount, bytes32 salt) external payable {
        require(
            keccak256(abi.encodePacked(amount, salt)) == commits[msg.sender],
            "Invalid reveal"
        );
        // 处理出价
    }
}

// ❌ 闪电贷攻击防御
// 关键: 不要在单笔交易中依赖价格预言机的即时值
contract SafePricing {
    // ✅ 使用 TWAP（时间加权平均价格）
    function getPrice() external view returns (uint256) {
        // 使用 Uniswap V3 TWAP Oracle
        // 而不是即时 spot price
    }
}
```

### 安全审计清单

| 检查项 | 严重性 | 工具 |
|--------|--------|------|
| 重入攻击 | Critical | Slither, Mythril |
| 整数溢出 | High | Solidity 0.8+ 内建 |
| 权限控制 | Critical | 手动审查 |
| 前置交易 | Medium | 架构设计 |
| 闪电贷攻击 | High | 手动审查 |
| 未检查返回值 | High | Slither |
| Gas 限制 DoS | Medium | 手动审查 |
| 时间戳依赖 | Low | Slither |
| 随机数不安全 | High | 手动审查 |
| 自毁漏洞 | Medium | Mythril |

---

## 测试框架

### 1. Foundry（推荐）

```solidity
// test/Vault.t.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/Vault.sol";

contract VaultTest is Test {
    Vault public vault;
    address public alice = makeAddr("alice");
    address public bob = makeAddr("bob");

    function setUp() public {
        vault = new Vault();
        vm.deal(alice, 100 ether);
        vm.deal(bob, 100 ether);
    }

    function test_Deposit() public {
        vm.prank(alice);
        vault.deposit{value: 1 ether}();

        assertEq(vault.balanceOf(alice), 1 ether);
        assertEq(vault.totalDeposits(), 1 ether);
    }

    function test_Withdraw() public {
        vm.startPrank(alice);
        vault.deposit{value: 5 ether}();
        vault.withdraw(2 ether);
        vm.stopPrank();

        assertEq(vault.balanceOf(alice), 3 ether);
    }

    function test_RevertWhen_InsufficientBalance() public {
        vm.prank(alice);
        vm.expectRevert(
            abi.encodeWithSelector(
                Vault.InsufficientBalance.selector,
                1 ether,
                0
            )
        );
        vault.withdraw(1 ether);
    }

    // Fuzz 测试
    function testFuzz_Deposit(uint256 amount) public {
        amount = bound(amount, 0.01 ether, 100 ether);

        vm.prank(alice);
        vault.deposit{value: amount}();

        assertEq(vault.balanceOf(alice), amount);
    }

    // 不变量测试
    function invariant_TotalDepositsMatchBalance() public view {
        assertEq(
            vault.totalDeposits(),
            address(vault).balance
        );
    }
}
```

**Foundry 命令**:
```bash
# 运行所有测试
forge test

# 详细输出
forge test -vvvv

# 运行单个测试
forge test --match-test test_Deposit

# Gas 报告
forge test --gas-report

# 覆盖率
forge coverage

# 部署
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast

# 验证合约
forge verify-contract $ADDRESS src/Vault.sol:Vault --etherscan-api-key $KEY
```

### 2. Hardhat

```javascript
// test/Vault.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("Vault", function () {
  async function deployFixture() {
    const [owner, alice, bob] = await ethers.getSigners();
    const Vault = await ethers.getContractFactory("Vault");
    const vault = await Vault.deploy();
    return { vault, owner, alice, bob };
  }

  describe("Deposit", function () {
    it("should accept deposits", async function () {
      const { vault, alice } = await loadFixture(deployFixture);

      await vault.connect(alice).deposit({
        value: ethers.parseEther("1.0")
      });

      expect(await vault.balanceOf(alice.address))
        .to.equal(ethers.parseEther("1.0"));
    });

    it("should emit Deposited event", async function () {
      const { vault, alice } = await loadFixture(deployFixture);

      await expect(
        vault.connect(alice).deposit({ value: ethers.parseEther("1.0") })
      ).to.emit(vault, "Deposited")
       .withArgs(alice.address, ethers.parseEther("1.0"));
    });

    it("should revert on zero deposit", async function () {
      const { vault, alice } = await loadFixture(deployFixture);

      await expect(
        vault.connect(alice).deposit({ value: 0 })
      ).to.be.revertedWithCustomError(vault, "DepositOutOfRange");
    });
  });

  describe("Withdraw", function () {
    it("should allow withdrawal", async function () {
      const { vault, alice } = await loadFixture(deployFixture);

      await vault.connect(alice).deposit({
        value: ethers.parseEther("5.0")
      });

      const balanceBefore = await ethers.provider.getBalance(alice.address);
      const tx = await vault.connect(alice).withdraw(ethers.parseEther("2.0"));
      const receipt = await tx.wait();
      const gasUsed = receipt.gasUsed * receipt.gasPrice;
      const balanceAfter = await ethers.provider.getBalance(alice.address);

      expect(balanceAfter - balanceBefore + gasUsed)
        .to.equal(ethers.parseEther("2.0"));
    });
  });
});
```

---

## DeFi 开发模式

### 1. AMM（自动做市商）

```solidity
// 简化的 Constant Product AMM (x * y = k)
contract SimpleAMM {
    IERC20 public tokenA;
    IERC20 public tokenB;
    uint256 public reserveA;
    uint256 public reserveB;

    uint256 public constant FEE_NUMERATOR = 3;
    uint256 public constant FEE_DENOMINATOR = 1000; // 0.3% fee

    function swap(address tokenIn, uint256 amountIn) external returns (uint256 amountOut) {
        require(amountIn > 0, "Zero amount");

        bool isTokenA = tokenIn == address(tokenA);
        (uint256 reserveIn, uint256 reserveOut) = isTokenA
            ? (reserveA, reserveB)
            : (reserveB, reserveA);

        // 扣除手续费
        uint256 amountInWithFee = amountIn * (FEE_DENOMINATOR - FEE_NUMERATOR);

        // x * y = k => amountOut = reserveOut * amountInWithFee / (reserveIn * 1000 + amountInWithFee)
        amountOut = (reserveOut * amountInWithFee) /
            (reserveIn * FEE_DENOMINATOR + amountInWithFee);

        // 更新储备
        if (isTokenA) {
            reserveA += amountIn;
            reserveB -= amountOut;
            tokenA.transferFrom(msg.sender, address(this), amountIn);
            tokenB.transfer(msg.sender, amountOut);
        } else {
            reserveB += amountIn;
            reserveA -= amountOut;
            tokenB.transferFrom(msg.sender, address(this), amountIn);
            tokenA.transfer(msg.sender, amountOut);
        }
    }
}
```

### 2. 借贷协议核心逻辑

```solidity
contract SimpleLending {
    struct Market {
        uint256 totalDeposits;
        uint256 totalBorrows;
        uint256 interestRate; // 年化利率 (basis points)
        uint256 collateralFactor; // 抵押率 (basis points, e.g., 7500 = 75%)
    }

    mapping(address => Market) public markets;
    mapping(address => mapping(address => uint256)) public deposits;
    mapping(address => mapping(address => uint256)) public borrows;

    function supply(address token, uint256 amount) external {
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        deposits[msg.sender][token] += amount;
        markets[token].totalDeposits += amount;
    }

    function borrow(address token, uint256 amount) external {
        // 检查抵押率
        uint256 collateralValue = getCollateralValue(msg.sender);
        uint256 borrowValue = getBorrowValue(msg.sender) + getTokenValue(token, amount);
        require(
            borrowValue * 10000 <= collateralValue * markets[token].collateralFactor,
            "Insufficient collateral"
        );

        borrows[msg.sender][token] += amount;
        markets[token].totalBorrows += amount;
        IERC20(token).transfer(msg.sender, amount);
    }
}
```

### 3. NFT 开发模式

```solidity
// ERC-721A 批量铸造（Gas 优化）
import "erc721a/contracts/ERC721A.sol";

contract OptimizedNFT is ERC721A {
    uint256 public constant MAX_SUPPLY = 10000;
    uint256 public constant MINT_PRICE = 0.08 ether;
    uint256 public constant MAX_PER_TX = 10;

    string private _baseTokenURI;
    bool public mintActive;

    constructor() ERC721A("OptimizedNFT", "ONFT") {}

    function mint(uint256 quantity) external payable {
        require(mintActive, "Mint not active");
        require(quantity <= MAX_PER_TX, "Exceeds max per tx");
        require(totalSupply() + quantity <= MAX_SUPPLY, "Exceeds supply");
        require(msg.value >= MINT_PRICE * quantity, "Insufficient payment");

        _mint(msg.sender, quantity);
    }

    // Merkle Tree 白名单
    bytes32 public merkleRoot;

    function whitelistMint(uint256 quantity, bytes32[] calldata proof)
        external
        payable
    {
        bytes32 leaf = keccak256(abi.encodePacked(msg.sender));
        require(
            MerkleProof.verify(proof, merkleRoot, leaf),
            "Not whitelisted"
        );
        _mint(msg.sender, quantity);
    }
}
```

### 4. DAO 治理模式

```solidity
import "@openzeppelin/contracts/governance/Governor.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorCountingSimple.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorVotes.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorTimelockControl.sol";

contract MyDAO is
    Governor,
    GovernorCountingSimple,
    GovernorVotes,
    GovernorTimelockControl
{
    constructor(
        IVotes _token,
        TimelockController _timelock
    )
        Governor("MyDAO")
        GovernorVotes(_token)
        GovernorTimelockControl(_timelock)
    {}

    function votingDelay() public pure override returns (uint256) {
        return 1 days;   // 投票延迟
    }

    function votingPeriod() public pure override returns (uint256) {
        return 1 weeks;  // 投票持续时间
    }

    function quorum(uint256) public pure override returns (uint256) {
        return 100_000e18;  // 法定人数（代币数量）
    }

    function proposalThreshold() public pure override returns (uint256) {
        return 1000e18;  // 提案门槛
    }
}
```

---

## Layer 2 解决方案

### 1. Rollup 概览

```
┌──────────────────────────────────────────────┐
│                Layer 1 (以太坊)                │
│  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Rollup 合约   │  │ 数据可用性层          │  │
│  │ (验证证明)    │  │ (存储交易数据)        │  │
│  └──────────────┘  └──────────────────────┘  │
├──────────────────────────────────────────────┤
│                Layer 2 (Rollup)               │
│  ┌──────────────┐  ┌──────────────────────┐  │
│  │ 排序器        │  │ 执行环境              │  │
│  │ (Sequencer)  │  │ (交易处理)            │  │
│  └──────────────┘  └──────────────────────┘  │
└──────────────────────────────────────────────┘
```

### 2. Optimistic Rollup vs ZK Rollup

| 维度 | Optimistic Rollup | ZK Rollup |
|------|-------------------|-----------|
| 代表项目 | Optimism, Arbitrum, Base | zkSync, StarkNet, Scroll |
| 验证机制 | 欺诈证明（挑战期 7 天） | 零知识证明（即时验证） |
| 提款时间 | 7 天（原生桥） | 分钟级 |
| Gas 成本 | 较低 | 中等（证明生成成本） |
| EVM 兼容性 | 高（几乎完全兼容） | 中-高（不断改善） |
| 适合场景 | 通用 DApp | 高频交易、支付 |
| 开发难度 | 低（与 L1 几乎相同） | 中（需了解 ZK 限制） |

### 3. Optimistic Rollup 开发（Optimism/Base）

```javascript
// 在 Optimism/Base 上部署与 L1 几乎相同
// hardhat.config.js
module.exports = {
  networks: {
    optimism: {
      url: "https://mainnet.optimism.io",
      chainId: 10,
      accounts: [process.env.PRIVATE_KEY]
    },
    base: {
      url: "https://mainnet.base.org",
      chainId: 8453,
      accounts: [process.env.PRIVATE_KEY]
    },
    arbitrum: {
      url: "https://arb1.arbitrum.io/rpc",
      chainId: 42161,
      accounts: [process.env.PRIVATE_KEY]
    }
  }
};

// L1 <-> L2 消息传递 (Optimism)
const { CrossChainMessenger } = require("@eth-optimism/sdk");

const messenger = new CrossChainMessenger({
  l1ChainId: 1,
  l2ChainId: 10,
  l1SignerOrProvider: l1Signer,
  l2SignerOrProvider: l2Signer,
});

// L1 -> L2 存款
await messenger.depositETH(ethers.parseEther("1.0"));

// L2 -> L1 提款（需等待挑战期）
await messenger.withdrawETH(ethers.parseEther("0.5"));
```

### 4. ZK Rollup 基础概念

```
零知识证明核心思想:
证明者可以向验证者证明自己知道某个信息，
而无需透露该信息本身。

ZK-SNARK: 简洁非交互式知识论证
- 证明大小: 恒定（~200 bytes）
- 验证时间: 恒定（~几毫秒）
- 需要可信设置（Trusted Setup）

ZK-STARK: 可扩展透明知识论证
- 证明大小: 更大（~KB 级）
- 验证时间: 对数级
- 无需可信设置
- 抗量子计算
```

**zkSync 开发示例**:
```bash
# 使用 zkSync CLI
npx zksync-cli create my-project --template hardhat_solidity

# 部署到 zkSync
npx hardhat deploy-zksync --script deploy.ts --network zkSyncTestnet
```

---

## 开发工具链

### 常用工具对比

| 工具 | 类型 | 语言 | 特点 |
|------|------|------|------|
| Foundry | 开发框架 | Solidity | 速度快、Solidity 原生测试 |
| Hardhat | 开发框架 | JavaScript | 生态丰富、插件多 |
| Remix | 在线 IDE | Solidity | 零配置、适合学习 |
| Slither | 静态分析 | Python | 漏洞检测、代码质量 |
| Mythril | 符号执行 | Python | 深度安全分析 |
| Tenderly | 调试/监控 | SaaS | 交易模拟、监控告警 |
| Etherscan | 区块浏览器 | SaaS | 合约验证、交易查看 |

### 安全分析命令

```bash
# Slither 静态分析
slither src/Vault.sol --solc-remaps "@openzeppelin=node_modules/@openzeppelin"

# Mythril 符号执行
myth analyze src/Vault.sol --solc-json mythril.config.json

# Foundry 模糊测试
forge test --fuzz-runs 10000

# 合约大小检查（24KB 限制）
forge build --sizes
```

---

## Agent Checklist

### 合约设计阶段
- [ ] 确认目标链和 EVM 兼容性
- [ ] 设计存储布局（变量打包优化）
- [ ] 确定升级策略（不可变 / UUPS / Transparent / Diamond）
- [ ] 权限模型设计（Ownable / AccessControl / 多签）
- [ ] 确认代币标准（ERC-20 / ERC-721 / ERC-1155）
- [ ] 设计紧急机制（Pausable / 时间锁）

### 开发阶段
- [ ] 使用 Solidity 0.8.20+（内建溢出检查）
- [ ] 遵循 Checks-Effects-Interactions 模式
- [ ] 使用 OpenZeppelin 标准库
- [ ] 所有外部调用使用 ReentrancyGuard
- [ ] Custom error 替代 require string
- [ ] 事件覆盖所有状态变更
- [ ] NatSpec 注释完整

### Gas 优化阶段
- [ ] 变量 slot 打包
- [ ] 使用 calldata 替代 memory（只读参数）
- [ ] 循环中缓存 storage 变量
- [ ] 使用 immutable/constant
- [ ] unchecked 用于已知安全的算术
- [ ] 批量操作减少交易数

### 安全审计阶段
- [ ] Slither 静态分析无高危告警
- [ ] Mythril 符号执行通过
- [ ] Foundry fuzz testing 覆盖边界条件
- [ ] 重入攻击测试
- [ ] 权限边界测试
- [ ] 整数边界测试
- [ ] 闪电贷攻击场景评估
- [ ] 前置交易风险评估

### 测试阶段
- [ ] 单元测试覆盖率 > 95%
- [ ] Fuzz 测试覆盖关键函数
- [ ] 不变量测试定义
- [ ] Fork 测试（使用主网状态）
- [ ] Gas 报告生成并优化
- [ ] 边界条件测试（0值、最大值、空地址）

### 部署阶段
- [ ] 测试网部署并验证
- [ ] 合约代码在 Etherscan 验证
- [ ] 多签或时间锁保护管理功能
- [ ] 前端集成测试通过
- [ ] 监控和告警配置（Tenderly / OpenZeppelin Defender）
- [ ] 应急响应计划准备

---

**知识ID**: `smart-contract-development`
**领域**: blockchain
**类型**: standards
**难度**: advanced
**质量分**: 93
**维护者**: blockchain-team@superdev.com
**最后更新**: 2026-03-28
