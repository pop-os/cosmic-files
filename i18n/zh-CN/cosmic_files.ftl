cosmic-files = COSMIC 文件管理器
empty-folder = 空文件夹
empty-folder-hidden = 空文件夹（包含隐藏项目）
no-results = 未找到结果
filesystem = 文件系统
home = 主页
networks = 网络
notification-in-progress = 文件操作正在进行中
trash = 回收站
recents = 最近访问
undo = 撤销
today = 今天
# Desktop view options
desktop-view-options = 桌面视图选项...
show-on-desktop = 在桌面显示
desktop-folder-content = 桌面文件夹内容
mounted-drives = 已装载驱动器
trash-folder-icon = 回收站图标
icon-size-and-spacing = 图标大小与间距
icon-size = 图标大小
grid-spacing = 网格间距
# List view
name = 名称
modified = 修改时间
trashed-on = 删除时间
size = 大小
# Progress footer
details = 详细信息
dismiss = 清除消息
operations-running =
    正在进行 { $running } { $running ->
        [one] 个操作
       *[other] 个操作
    }（{ $percent }%）...
operations-running-finished =
    正在进行 { $running } { $running ->
        [one] 个操作
       *[other] 个操作
    }（{ $percent }%），{ $finished } 个操作已完成…
pause = 暂停
resume = 继续

# Dialogs


## Compress Dialog

create-archive = 创建压缩包

## Extract Dialog

extract-password-required = 需要密码
extract-to = 提取到...
extract-to-title = 提取到文件夹

## Empty Trash Dialog

empty-trash = 清空回收站
empty-trash-warning = 回收站中的所有内容会被永久删除

## Mount Error Dialog

mount-error = 无法访问驱动器

## New File/Folder Dialog

create-new-file = 新建文件
create-new-folder = 新建文件夹
file-name = 文件名称
folder-name = 文件夹名称
file-already-exists = 同名文件已存在
folder-already-exists = 同名文件夹已存在
name-hidden = 以 “.” 开头的文件会被隐藏
name-invalid = 名称不能是 “{ $filename }”
name-no-slashes = 名称不可以包含斜线

## Open/Save Dialog

cancel = 取消
create = 创建
open = 打开
open-file = 打开文件
open-folder = 打开文件夹
open-in-new-tab = 在新标签中打开
open-in-new-window = 在新窗口中打开
open-item-location = 打开项目位置
open-multiple-files = 打开多个文件
open-multiple-folders = 打开多个文件夹
save = 保存
save-file = 保存文件

## Open With Dialog

open-with-title = 您想要如何打开 “{ $name }”？
browse-store = 浏览 { $store }
other-apps = 其他应用程序
related-apps = 相关应用程序

## Permanently delete Dialog

selected-items = 选中的 { $items } 个项目
permanently-delete-question = 永久删除？
delete = 删除
permanently-delete-warning = { $target } 将被永久删除。此操作无法撤销。

## Rename Dialog

rename-file = 重命名文件
rename-folder = 重命名文件夹

## Replace Dialog

replace = 替换
replace-title = “{ $filename }” 已存在于该位置
replace-warning = 您想要使用您现在正在保存的文件替换掉它吗？一旦替换将会覆盖其内容。
replace-warning-operation = 您想要替换掉它吗？一旦替换将会覆盖其内容。
original-file = 原始文件
replace-with = 替换为
apply-to-all = 全部应用
keep-both = 保留两者
skip = 跳过

## Set as Executable and Launch Dialog

set-executable-and-launch = 设置为可执行文件并启动
set-executable-and-launch-description = 您想要将 “{ $name }” 设置为可执行文件并启动它吗？
set-and-launch = 设置并启动

## Metadata Dialog

open-with = 打开方式
owner = 所有者
group = 用户组
other = 其他用户

### Mode 0

none = 无

### Mode 1 (unusual)

execute-only = 仅执行

### Mode 2 (unusual)

write-only = 仅写入

### Mode 3 (unusual)

write-execute = 写入和执行

### Mode 4

read-only = 只读

### Mode 5

read-execute = 读取和执行

### Mode 6

read-write = 读取和写入

### Mode 7

read-write-execute = 读取、写入和执行

## Favorite Path Error Dialog

favorite-path-error = 打开路径时出错
favorite-path-error-description =
    无法打开 "{ $path }" 。
    "{ $path }" 可能不存在或您没有权限打开它。

    您想要从侧边栏中移除它吗？
remove = 移除
keep = 保留

# Context Pages


## About


## Add Network Drive

add-network-drive = 添加网络驱动器
connect = 连接
connect-anonymously = 匿名连接
connecting = 正在连接…
domain = 网络域
enter-server-address = 输入服务器地址
network-drive-description =
    服务器地址包含协议前缀和地址。
    示例: ssh://192.168.0.1，ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    可用协议，前缀
    AppleTalk，afp://
    文件传输协议，ftp:// 或者 ftps://
    网络文件系统，nfs://
    服务器消息块，smb://
    SSH 文件传输协议，sftp:// 或者 ssh://
    WebDav，dav:// 或者 davs://
network-drive-error = 无法访问网络驱动器
password = 密码
remember-password = 记住密码
try-again = 重试
username = 用户名

## Operations

cancelled = 已取消
edit-history = 编辑历史记录
history = 历史记录
no-history = 历史记录为空。
pending = 待处理
progress = { $percent }%
progress-cancelled = { $percent }%，已取消
progress-paused = { $percent }%，已暂停
failed = 失败
complete = 完成
compressing =
    正在从“{ $from }”压缩 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }到“{ $to }”（{ $progress }）…
compressed =
    已从“{ $from }”压缩 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }到“{ $to }”
copy_noun = 复制
creating = 正在“{ $parent }”里创建“{ $name }”
created = 已在“{ $parent }”里创建“{ $name }”
copying =
    正在从“{ $from }”复制 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }到“{ $to }”（{ $progress }）…
copied =
    已从“{ $from }”复制 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }到“{ $to }”
deleting =
    正在从{ trash }删除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }（{ $progress }）…
deleted =
    已从{ trash }删除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
emptying-trash = 正在清空{ trash }（{ $progress }）…
emptied-trash = 已清空{ trash }
extracting =
    正在从“{ $from }”提取{ $items } { $items ->
        [one] 项目
       *[other] 项目
    }到“{ $to }”（{ $progress }）…
extracted =
    已从“{ $from }”提取 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }到“{ $to }”
setting-executable-and-launching = 设置 "{ $name }" 为可执行文件并启动
set-executable-and-launched = 已设 "{ $name }" 为可执行文件并且启动
setting-permissions = 正在设置 "{ $name }" 的权限为 { $mode }
set-permissions = 设置 "{ $name }" 的权限为 { $mode }
moving =
    正在从“{ $from }”移动 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }到“{ $to }”（{ $progress }）…
moved =
    已从“{ $from }”移动 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }到“{ $to }”
permanently-deleting =
    正在永久删除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
permanently-deleted =
    已永久删除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
removing-from-recents =
    正在从{ recents }中移除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
removed-from-recents =
    已从{ recents }中移除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
renaming = 正在将“{ $from }”重命名为“{ $to }”
renamed = 已重命名"{ $from }"为"{ $to }"
restoring =
    正在从{ trash }中还原 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }（{ $progress }）…
restored =
    已从{ trash }中还原 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
unknown-folder = 未知文件夹

## Open with

menu-open-with = 打开方式...
default-app = { $name }（默认）

## Show details

show-details = 显示详情
type = 文件类型：{ $mime }
items = 文件数：{ $items }
item-size = 文件大小：{ $size }
item-created = 创建于：{ $created }
item-modified = 修改于：{ $modified }
item-accessed = 访问于：{ $accessed }
calculating = 计算中...

## Settings

settings = 设置
single-click = 单击打开

### Appearance

appearance = 外观
theme = 主题
match-desktop = 匹配桌面
dark = 暗色模式
light = 亮色模式

### Type to Search

type-to-search = 输入即可搜索
type-to-search-recursive = 搜索当前文件夹及其所有子文件夹
type-to-search-enter-path = 输入文件夹或文件路径
# Context menu
add-to-sidebar = 加入侧边栏
compress = 压缩
delete-permanently = 永久删除
eject = 弹出
extract-here = 解压到此处
new-file = 新建文件...
new-folder = 新建文件夹...
open-in-terminal = 在终端模拟器中打开
move-to-trash = 移动到回收站
restore-from-trash = 从回收站中还原
remove-from-sidebar = 从侧边栏中移除
sort-by-name = 按名称排序
sort-by-modified = 按修改时间排序
sort-by-size = 按文件大小排序
sort-by-trashed = 按删除时间排序
remove-from-recents = 从最近访问中移除

## Desktop

change-wallpaper = 更改壁纸...
desktop-appearance = 桌面外观...
display-settings = 显示设置...

# Menu


## File

file = 文件
new-tab = 新建标签
new-window = 新建窗口
reload-folder = 刷新文件夹
rename = 重命名...
close-tab = 关闭标签
quit = 退出

## Edit

edit = 编辑
cut = 剪切
copy = 复制
paste = 粘贴
select-all = 全选

## View

zoom-in = 增大
default-size = 默认大小
zoom-out = 缩小
view = 视图
grid-view = 表格视图
list-view = 列表视图
show-hidden-files = 显示隐藏文件
list-directories-first = 优先列出目录
gallery-preview = 图库预览
menu-settings = 设置...
menu-about = 关于 COSMIC 文件…

## Sort

sort = 排序
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = 最新优先
sort-oldest-first = 最旧优先
sort-smallest-to-largest = 从小到大
sort-largest-to-smallest = 从大到小
repository = 仓库
support = 支持
progress-failed = { $percent }%，失败
empty-trash-title = 确定要清空回收站？
type-to-search-select = 选择第一个匹配的文件或文件夹
pasted-video = 已粘贴视频
pasted-image = 已粘贴图片
pasted-text = 已粘贴文本
