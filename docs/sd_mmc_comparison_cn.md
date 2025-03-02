# SD卡和MMC卡结构的比较

## 共同元素

`sd_card_t`和`mmc_card_t`结构体共享以下字段：

| 字段 | 描述 |
|-------|-------------|
| `host` | 主机配置指针（不同类型） |
| `isHostReady` | 指示是否需要主机重新初始化的标志 |
| `noInteralAlign` | 启用/禁用缓冲区对齐功能的标志 |
| `busClock_Hz` | 总线时钟频率（赫兹） |
| `relativeAddress` | 卡的相对地址 |
| `flags` | 能力标志 |
| `internalBuffer` | 内部缓冲区指针 |
| `internalBufferSize` | 内部缓冲区大小 |
| `ocr` | 原始OCR（操作条件寄存器）内容 |
| `blockSize` | 卡块大小 |
| `lock` | 卡访问互斥锁 |

## 特定字段

### SD卡特有字段（`sd_card_t`）

| 字段 | 类型 | 描述 |
|-------|------|-------------|
| `usrParam` | `sd_usr_param_t` | SD用户参数 |
| `version` | `uint32_t` | 卡版本 |
| `cid` | `sd_cid_t` | 卡识别数据 |
| `csd` | `sd_csd_t` | 卡特定数据 |
| `scr` | `sd_scr_t` | SD配置寄存器 |
| `stat` | `sd_status_t` | SD 512位状态 |
| `blockCount` | `uint32_t` | 总块数 |
| `currentTiming` | `sd_timing_mode_t` | 当前时序模式 |
| `driverStrength` | `sd_driver_strength_t` | 驱动器强度 |
| `maxCurrent` | `sd_max_current_t` | 卡电流限制 |
| `operationVoltage` | `sdmmc_operation_voltage_t` | 卡操作电压 |

### MMC卡特有字段（`mmc_card_t`）

| 字段 | 类型 | 描述 |
|-------|------|-------------|
| `usrParam` | `mmc_usr_param_t` | MMC用户参数 |
| `enablePreDefinedBlockCount` | `bool` | 启用读/写预定义块计数 |
| `cid` | `mmc_cid_t` | 卡识别数据 |
| `csd` | `mmc_csd_t` | 卡特定数据 |
| `extendedCsd` | `mmc_extended_csd_t` | 扩展卡特定数据 |
| `userPartitionBlocks` | `uint32_t` | 用户分区中的总块数 |
| `bootPartitionBlocks` | `uint32_t` | 引导分区大小（以块为单位） |
| `eraseGroupBlocks` | `uint32_t` | 擦除组大小（以块为单位） |
| `currentPartition` | `mmc_access_partition_t` | 当前访问分区 |
| `hostVoltageWindowVCCQ` | `mmc_voltage_window_t` | VCCQ的主机电压窗口 |
| `hostVoltageWindowVCC` | `mmc_voltage_window_t` | VCC的主机电压窗口 |
| `busTiming` | `mmc_high_speed_timing_t` | 当前总线时序模式 |
| `busWidth` | `mmc_data_bus_width_t` | 当前数据总线宽度 |

## 主要结构差异

1. **分区管理**：
   - MMC卡具有分区管理功能（`currentPartition`、`userPartitionBlocks`、`bootPartitionBlocks`）
   - SD卡使用更简单的结构，只有单一的`blockCount`

2. **配置数据**：
   - SD卡有`scr`（SD配置寄存器）
   - MMC卡有`extendedCsd`（扩展卡特定数据）

3. **电压和总线管理**：
   - MMC卡具有更详细的电压控制（`hostVoltageWindowVCCQ`、`hostVoltageWindowVCC`）
   - MMC卡明确管理总线宽度（`busWidth`）

4. **块管理**：
   - MMC有`enablePreDefinedBlockCount`和`eraseGroupBlocks`
   - SD使用更简单的块计数方法