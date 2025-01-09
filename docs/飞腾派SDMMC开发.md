# 前言

现在对于飞腾派的`arceos`的开发是非常的成熟的,但是对于一个小白是非常麻烦的.

> 这里一定要注意可以跟踪驱动的开发进程,也就是看`commit`的过程

```shell
 * 1.0   zhugengyu  2021/12/2    init
 * 1.1   zhugengyu  2022/6/6     modify according to tech manual.
 * 1.2   zhugengyu  2022/7/15    adopt to e2000
 * 1.3   zhugengyu  2022/11/23   fix multi-block rw issues
 * 2.0   zhugengyu  2023/9/16    rename as sdif, support SD 3.0 and rework clock timing
 * 2.1   zhugengyu  2023/10/23   add sdio interrupt handler
```

> 这里应该选取所有文件都在的`V1.1`版本.这个版本在总的`V0.2.0`里边

[V0.2.0版本](https://gitee.com/phytium_embedded/phytium-standalone-sdk/tree/v0.2.0/)

# 资源

[飞腾派数据手册V1.0版本](https://github.com/arceos-usb/arceos_experiment/blob/usb-camera-base/doc/resources/%E9%A3%9E%E8%85%BE%E6%B4%BE%E6%95%B0%E6%8D%AE%E6%89%8B%E5%86%8CV1.0%E7%89%88%E6%9C%AC.pdf)

萤火工场·CEK8903飞腾派软件开发手册-V1.01(待上传)

- [ ] todo 

[飞腾派软件编程手册V1.0](https://github.com/arceos-usb/arceos_experiment/blob/usb-camera-base/doc/resources/%E9%A3%9E%E8%85%BE%E6%B4%BE%E8%BD%AF%E4%BB%B6%E7%BC%96%E7%A8%8B%E6%89%8B%E5%86%8CV1.0.pdf)

[Phytium-Standalone-SDK:](https://gitee.com/phytium_embedded/phytium-standalone-sdk)

[提交 · Phytium嵌入式软件](https://gitee.com/phytium_embedded/phytium-standalone-sdk/commits/master)

# blogs 

[Linux MMC 驱动子系统详解 - Buttering's Blog](https://buttering.github.io/EasyBlog/2023/02/07/Linux%20MMC%20%E9%A9%B1%E5%8A%A8%E5%AD%90%E7%B3%BB%E7%BB%9F%E8%AF%A6%E8%A7%A3/)

[【MMC子系统】一、MMC_SD_SDIO介绍 | Donge Blog](https://uniondong.github.io/docs/linux/linux_mmc_subsystem/mmc%E5%AD%90%E7%B3%BB%E7%BB%9F%E4%B8%80mmc_sd_sdio%E4%BB%8B%E7%BB%8D/)

[MMC/SD/SDIO介绍](http://www.wowotech.net/basic_tech/mmc_sd_sdio_intro.html)


# 已有工作解读

## 预备知识

**萤火工场·CEK8903飞腾派软件开发手册-V1.01/6.高手进阶/高手进阶**

### 交叉编译环境搭建

- [ ] todo

### `make disk_img`指令生成

引用了`dosfstools`工具,这个工具是Ubuntu预装的.

### 烧录工具

1. SD卡
2. 读卡器
3. CH340-USB-TTL

### phytium飞腾派SDK

[fsdif.md · Phytium嵌入式软件](https://gitee.com/phytium_embedded/phytium-standalone-sdk/blob/master/doc/reference/driver/fsdif.md)



# 可能需要

`gdb`调试

