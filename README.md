# Current Status
![img](/screenshots/1.png)

# Arrangement
## TODO
- [ ] 读取图片/文件夹
- [ ] 查看的选项
  - [ ] 旋转
  - [ ] 放缩
  - [ ] 文件信息
  - [ ] 上一个/下一个图片
  - [ ] 全部图片视图
- [ ] 基本的编辑功能
  - [ ] 剪裁
  - [ ] 旋转、翻转
  - [ ] 画笔
- [ ] 可能加入的功能
  - [ ] 滤镜
  - [ ] 水印
  - [ ] 几何图形
  - [ ] 消除
  - [ ] 以及各种花里胡哨的
## LATER

- [ ] 用户体验
  - [ ] 启动、图片切换、加载等的动画
  - [ ] 保存会话记录
  - [ ] 批量处理
  - [ ] 可更改的软件设置
- [ ] 优化？
  - [ ] 异步处理
  - [ ] 多线程
  - [ ] GPU加速
  - [ ] 减小app体积（刚开始就已经14MB了）
  - [ ] portable

# Build(Dev)

## windows 10

- C++ build Tools
- Node.js v17.3.0
- yarn v1.22.15
- webview2 v96.0.1054.62
- rustc v1.59.0-nightly
- cargo v1.59.0-nightly

```sh
yarn tauri dev
```

## Linux(tested on ArchWsl)