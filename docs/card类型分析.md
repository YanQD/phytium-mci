# 结构体分析: _sdio_card, _mmc_card 和 _sd_card

我将对这三个结构体进行详细分析，比较它们的相同点和不同点。

## 相同点

1. **基础主机信息**:

   - 所有三个结构体都包含 `sdmmchost_t *host` 成员，用于存储主机信息
   - 都有 `isHostReady` 标志，用于指示是否需要重新初始化主机
2. **内存缓冲区**:

   - 都包含内部缓冲区指针 `internalBuffer` 和对应的大小 `internalBufferSize`（_sdio_card 和 _mmc_card 使用 `internalBufferSize`，而 _sd_card 也有类似字段）
   - 都有禁用内部对齐的标志（`noInternalAlign`/_mmc_card用 `noInteralAlign`/_sd_card用 `noInteralAlign`）
3. **总线和地址信息**:

   - 都包含 `busClock_Hz` 用于指定总线时钟频率
   - 都包含 `relativeAddress` 表示卡的相对地址
4. **互斥锁**:

   - 所有结构体都包含 `sdmmc_osa_mutex_t lock` 用于卡访问锁定
5. **用户参数**:

   - 都有用户参数结构，分别为 `sdio_usr_param_t usrParam`, `mmc_usr_param_t usrParam` 和 `sd_usr_param_t usrParam`
6. **块信息**:

   - _mmc_card 和 _sd_card 都包含 `blockSize` 成员

## 不同点

1. **卡特有功能**:

   **SDIO 卡特有**:

   - SDIO版本信息 (`sdioVersion`, `cccrVersioin`)
   - IO功能相关字段 (`ioTotalNumber`, `io0blockSize`, `ioFBR`, `funcCIS`, `ioIRQHandler`, `ioIntIndex`, `ioIntNums`)
   - 共同CIS指针和表 (`commonCISPointer`, `commonCIS`)
   - 内存存在标志 (`memPresentFlag`)

   **MMC 卡特有**:

   - 预定义块计数标志 (`enablePreDefinedBlockCount`)
   - 扩展CSD (`extendedCsd`)
   - 分区特有字段 (`userPartitionBlocks`, `bootPartitionBlocks`, `eraseGroupBlocks`, `currentPartition`)
   - 电压窗口设置 (`hostVoltageWindowVCCQ`, `hostVoltageWindowVCC`)
   - 总线时序和宽度 (`busTiming`, `busWidth`)

   **SD 卡特有**:

   - SD版本 (`version` - 与_sdio_card中的 `sdVersion`不同)
   - SD状态 (`stat` - 512位状态)
   - 块总数 (`blockCount`)
2. **标识信息**:

   - _mmc_card 和 _sd_card 包含 CID, CSD 信息，而 _sdio_card 没有
   - _sd_card 特有 SCR (SD Configuration Register)
   - 三个结构体中的 OCR (Operation Conditions Register) 实现不同
3. **操作模式和电压**:

   - _sdio_card 和 _sd_card 共享一些相同的定时模式、驱动强度和电流限制参数，而 _mmc_card 使用不同的实现
4. **数据结构大小和复杂性**:

   - _sdio_card 结构体关注IO功能和中断处理
   - _mmc_card 结构体更关注分区和电压控制
   - _sd_card 结构体更简单，聚焦于基本存储功能
5. **标志位管理**:

   - _sdio_card 使用 `cccrflags`
   - _mmc_card 和 _sd_card 使用 `flags`，但它们的标志位定义可能不同

# 对比表格

| 字段/功能              | _sdio_card                                                               | _mmc_card                                                                                     | _sd_card                                       |
| ---------------------- | ------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| **主机信息**     | `sdmmchost_t *host`                                                    | `sdmmchost_t *host`                                                                         | `sdmmchost_t *host`                          |
| **用户参数**     | `sdio_usr_param_t usrParam`                                            | `mmc_usr_param_t usrParam`                                                                  | `sd_usr_param_t usrParam`                    |
| **主机就绪标志** | `bool isHostReady`                                                     | `bool isHostReady`                                                                          | `bool isHostReady`                           |
| **内存对齐控制** | `bool noInternalAlign`                                                 | `bool noInteralAlign`                                                                       | `bool noInteralAlign`                        |
| **内部缓冲区**   | `uint8_t *internalBuffer`                                              | `uint8_t *internalBuffer`                                                                   | `uint8_t *internalBuffer`                    |
| **缓冲区大小**   | `size_t internalBufferSize`                                            | `size_t internalBufferSize`                                                                 | `size_t internalBufferSize`                  |
| **总线时钟**     | `uint32_t busClock_Hz`                                                 | `uint32_t busClock_Hz`                                                                      | `uint32_t busClock_Hz`                       |
| **相对地址**     | `uint32_t relativeAddress`                                             | `uint32_t relativeAddress`                                                                  | `uint32_t relativeAddress`                   |
| **互斥锁**       | `sdmmc_osa_mutex_t lock`                                               | `sdmmc_osa_mutex_t lock`                                                                    | `sdmmc_osa_mutex_t lock`                     |
| **OCR寄存器**    | `uint32_t ocr` (24位可用)                                              | `uint32_t ocr`                                                                              | `uint32_t ocr`                               |
| **标志位**       | `uint32_t cccrflags`                                                   | `uint32_t flags`                                                                            | `uint32_t flags`                             |
| **版本信息**     | `uint8_t sdVersion<br>``uint8_t sdioVersion<br>``uint8_t cccrVersioin` | -                                                                                             | `uint32_t version`                           |
| **块大小**       | `uint32_t io0blockSize`                                                | `uint32_t blockSize`                                                                        | `uint32_t blockSize`                         |
| **块数量**       | -                                                                        | `uint32_t userPartitionBlocks`                                                              | `uint32_t blockCount`                        |
| **时序模式**     | `sd_timing_mode_t currentTiming`                                       | `mmc_high_speed_timing_t busTiming`                                                         | `sd_timing_mode_t currentTiming`             |
| **驱动强度**     | `sd_driver_strength_t driverStrength`                                  | -                                                                                             | `sd_driver_strength_t driverStrength`        |
| **最大电流**     | `sd_max_current_t maxCurrent`                                          | -                                                                                             | `sd_max_current_t maxCurrent`                |
| **操作电压**     | `sdmmc_operation_voltage_t operationVoltage`                           | -                                                                                             | `sdmmc_operation_voltage_t operationVoltage` |
| **卡识别数据**   | -                                                                        | `mmc_cid_t cid`                                                                             | `sd_cid_t cid`                               |
| **卡特定数据**   | -                                                                        | `mmc_csd_t csd`                                                                             | `sd_csd_t csd`                               |
| **SD配置寄存器** | -                                                                        | -                                                                                             | `sd_scr_t scr`                               |
| **SD状态**       | -                                                                        | -                                                                                             | `sd_status_t stat`                           |
| **扩展CSD**      | -                                                                        | `mmc_extended_csd_t extendedCsd`                                                            | -                                              |
| **分区块**       | -                                                                        | `uint32_t bootPartitionBlocks<br>``uint32_t eraseGroupBlocks`                               | -                                              |
| **当前分区**     | -                                                                        | `mmc_access_partition_t currentPartition`                                                   | -                                              |
| **总线宽度**     | -                                                                        | `mmc_data_bus_width_t busWidth`                                                             | -                                              |
| **电压窗口**     | -                                                                        | `mmc_voltage_window_t hostVoltageWindowVCCQ<br>``mmc_voltage_window_t hostVoltageWindowVCC` | -                                              |
| **预定义块计数** | -                                                                        | `bool enablePreDefinedBlockCount`                                                           | -                                              |
| **IO功能数量**   | `uint8_t ioTotalNumber`                                                | -                                                                                             | -                                              |
| **共同CIS指针**  | `uint32_t commonCISPointer`                                            | -                                                                                             | -                                              |
| **共同CIS表**    | `sdio_common_cis_t commonCIS`                                          | -                                                                                             | -                                              |
| **FBR表**        | `sdio_fbr_t ioFBR[FSL_SDIO_MAX_IO_NUMS]`                               | -                                                                                             | -                                              |
| **功能CIS表**    | `sdio_func_cis_t funcCIS[FSL_SDIO_MAX_IO_NUMS]`                        | -                                                                                             | -                                              |
| **IO中断处理**   | `sdio_io_irq_handler_t ioIRQHandler[FSL_SDIO_MAX_IO_NUMS]`             | -                                                                                             | -                                              |
| **IO中断索引**   | `uint8_t ioIntIndex`                                                   | -                                                                                             | -                                              |
| **IO中断数量**   | `uint8_t ioIntNums`                                                    | -                                                                                             | -                                              |
| **内存存在标志** | `bool memPresentFlag`                                                  | -                                                                                             | -                                              |

## 总结

这三个结构体代表了不同类型的存储卡：SD卡、MMC卡和SDIO卡。它们共享通用的主机接口和基础功能（如总线时钟、相对地址、内部缓冲区等），但在特定功能上有显著差异：

- **_sdio_card**: 主要关注IO功能、中断处理和通信能力，包含多个IO功能和中断处理程序
- **_mmc_card**: 更关注存储分区、电压控制和总线配置
- **_sd_card**: 相对更简单，主要关注基本存储功能和SD特定配置

这些结构体似乎是嵌入式系统中用于SD/MMC/SDIO卡控制的底层驱动程序实现的一部分。每个结构体针对特定卡类型进行了优化，以提供最适合其特性和功能的接口。
