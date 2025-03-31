# 改动记录

## 基本函数体方法和数据结构

* 为`MCI`添加`desc_list: FSdifIdmaDescList`字段，实现new方法

* 为`MCI`添加`cur_cmd: Option<MCICmdData>`字段，实现new方法。还没确定用Option还是指针

* 为`SDIFDevPIO`添加`rw_desc: *mut FSdifIdmaDesc`字段

* 修改`FSdifIDmaDescList::first_desc_dma`为usize类型；修改`MCIData::buf_dma`为usize类型，对应修改相关函数的定义

* 为MCIData添加buf_dma字段，用于记录dma传输的地址，相应在covert_command_info配置out_data的buf_dma

* mci/mod.rs添加dma相关函数

  * `pub fn set_idma_list(&mut self, desc: *mut FSdifIDmaDesc, desc_dma: u32, desc_num: u32) -> MCIResult`
  * `pub fn dma_transfer(&mut self, cmd_data: &mut MCICmdData) -> MCIResult`
  * `pub fn poll_wait_dma_end(&mut self, cmd_data: &mut MCICmdData) -> MCIResult`

* 剩余dma相关代码放到`mci_dma.rs`

  * `pub fn dump_dma_descriptor(&self, *desc_in_use*: u32)`
  * `pub(crate) fn setup_dma_descriptor(&mut self, *data*: &MCIData) -> MCIResult`
  * `pub(crate) fn dma_transfer_data(&mut self, *data*: &MCIData) -> MCIResult`

  

* commit 532156：

  1. 为`MCIHostDevice`添加type_id()，为了得到host的dev字段（添加`MCIHost::get_dev()`)。貌似可以用其他方法得到，目前先不修改
  2. 修改`SDIFDevPIO`的`do_init()`方法，无论是否启用dma均设置`dma_list`，后面还要修改
  3. 修改`MCI`的`restart`方法，可以根据传入的`MCIId`选择`restart`返回的类型
  4. 为`SdCard`添加`dma_rw_init()`方法，负责设备初始化完毕后启用dma传输之前的操作

* commit 168ae5

  1. dma不能直接用buffer，尝试用全局变量写内存池
  1. 修改convert_command_info把rx_data take出来，让buf_dma使用transfer_function传入的buffer地址

* commit

	1. 修改dsb()，添加isb汇编指令
	2. 修改descriptor_set()，MCIDescListAddr地址寄存器分高位和低位
	3. convert_cmd_info结尾添加dsb刷新cache，好像会和其他地方重复
	4. transfer_function中poll_end结束后强制dsb

## 初始化

* 为SDIFDevPIO::new()添加了desc_num字段，以便创建SDIFDevPIO实例时为rw_desc分配空间

## 其他

调整了部分代码格式，规范了部分函数的注释格式