cosmic-files = COSMIC 檔案
empty-folder = 空資料夾
empty-folder-hidden = 空資料夾（包含隱藏項目）
no-results = 找不到結果
filesystem = 檔案系統
home = 家目錄
networks = 網路
notification-in-progress = 檔案操作正在進行中
trash = 垃圾桶
recents = 最近使用
undo = 復原
today = 今天
# List view
name = 名稱
modified = 修改日期
size = 大小

# Dialogs


## Compress Dialog

create-archive = 建立壓縮檔案

## Empty Trash Dialog

empty-trash = 清空垃圾桶
empty-trash-warning = 垃圾桶中的項目將被永久刪除

## New File/Folder Dialog

create-new-file = 建立新檔案
create-new-folder = 建立新資料夾
file-name = 檔案名稱
folder-name = 資料夾名稱
file-already-exists = 相同名稱的檔案已經存在
folder-already-exists = 相同名稱的資料夾已經存在
name-hidden = 以「.」開頭的名稱將會被隱藏
name-invalid = 名稱不能是「{ $filename }」
name-no-slashes = 名稱不能包含斜線

## Open/Save Dialog

cancel = 取消
create = 創作
open = 開啟
open-file = 開啟檔案
open-folder = 開啟資料夾
open-in-new-tab = 在新分頁中開啟
open-in-new-window = 在新視窗中開啟
open-item-location = 開啟項目位置
open-multiple-files = 開啟多個檔案
open-multiple-folders = 開啟多個資料夾
save = 儲存
save-file = 儲存檔案

## Rename Dialog

rename-file = 重新命名檔案
rename-folder = 重新命名資料夾

## Replace Dialog

replace = 取代
replace-title = 「{ $filename }」已存在於此位置
replace-warning = 你要取代它嗎？取代將覆蓋其內容。
replace-warning-operation = 你要取代它嗎？取代將覆蓋其內容。
original-file = 原始檔案
replace-with = 取代為
apply-to-all = 套用至全部
keep-both = 保留兩者
skip = 跳過

## Metadata Dialog

owner = 擁有者
group = 群組
other = 其他

# Context Pages


## About


## Add Network Drive

add-network-drive = 新增網路磁碟機
connect = 連線
connect-anonymously = 匿名連線
connecting = 連線中...
domain = 網域
enter-server-address = 輸入伺服器地址
network-drive-description =
    伺服器地址包括協定前綴和地址。
    範例：ssh://192.168.0.1, ftp://[2001:db8::1]
network-drive-schemes =
    可用協定,前綴
    AppleTalk,afp://
    檔案傳輸協定,ftp:// 或 ftps://
    網路檔案系統,nfs://
    伺服器訊息區塊,smb://
    SSH 檔案傳輸協定,sftp:// 或 ssh://
    WebDav,dav:// 或 davs://
network-drive-error = 無法存取網路磁碟機
password = 密碼
remember-password = 記住密碼
try-again = 再試一次
username = 使用者名稱

## Operations

edit-history = 編輯歷史
history = 歷史紀錄
no-history = 無歷史記錄項目。
pending = 待處理
failed = 失敗
complete = 完成
compressing =
    正在壓縮 { $items } { $items ->
        [one] 項目
       *[other] 項目
    } 從「{ $from }」到 「{ $to }」（{ $progress }）...
compressed =
    已壓縮 { $items } { $items ->
        [one] 項目
       *[other] 項目
    }從「{ $from }」到「{ $to }」
copy_noun = 複製
creating = 正在建立「{ $name }」於「{ $parent }」
created = 已建立「{ $name }」於「{ $parent }」
copying =
    正在複製 { $items } { $items ->
        [one] 項目
       *[other] 項目
    }從「{ $from }」到「{ $to }」（{ $progress }）...
copied =
    已複製 { $items } { $items ->
        [one] 項目
       *[other] 項目
    }從「{ $from }」到「{ $to }」
emptying-trash = 正在清空 { trash }（{ $progress }）…
emptied-trash = 已經清空 { trash }
extracting =
    正在解壓縮 { $items } 項目 { $items ->
        [one] 項目
       *[other] 項目
    } 從「{ $from }」到「{ $to }」（{ $progress }）...
extracted =
    已解壓縮 { $items } 項目 { $items ->
        [one] 項目
       *[other] 項目
    } 從 { $from } 到 { $to }
moving =
    正在移動 { $items } { $items ->
        [one] 項目
       *[other] 項目
    } 從「{ $from }」到「{ $to }」（{ $progress }）...
moved =
    已移動 { $items } { $items ->
        [one] 項目
       *[other] 項目
    } 從「{ $from }」到「{ $to }」
renaming = 正在重新命名「{ $from }」為「{ $to }」
renamed = 已重新命名 { $from } 為 { $to }
restoring =
    正在還原 { $items } 項目 { $items ->
        [one] 項目
       *[other] 項目
    } 從{ trash }（{ $progress }）...
restored =
    已還原 { $items } 項目 { $items ->
        [one] 項目
       *[other] 項目
    } 從{ trash }
unknown-folder = 未知資料夾

## Open with

menu-open-with = 開啟方式...
default-app = { $name } （預設）

## Show details

show-details = 顯示詳細資料

## Settings

settings = 設定

### Appearance

appearance = 外觀
theme = 主題
match-desktop = 符合桌面
dark = 暗色
light = 亮色
# Context menu
add-to-sidebar = 加入側邊欄
compress = 壓縮
extract-here = 解壓縮至此
new-file = 新檔案...
new-folder = 新資料夾...
open-in-terminal = 在終端機中開啟
move-to-trash = 移動至垃圾桶
restore-from-trash = 從垃圾桶還原
remove-from-sidebar = 從側邊欄移除
sort-by-name = 依名稱排序
sort-by-modified = 依修改日期排序
sort-by-size = 依大小排序

# Menu


## File

file = 檔案
new-tab = 新建分頁
new-window = 新建視窗
rename = 重新命名...
close-tab = 關閉分頁
quit = 退出

## Edit

edit = 編輯
cut = 剪下
copy = 複製
paste = 貼上
select-all = 全選

## View

zoom-in = 放大
default-size = 預設大小
zoom-out = 縮小
view = 檢視
grid-view = 網格檢視
list-view = 列表檢視
show-hidden-files = 顯示隱藏檔案
list-directories-first = 優先列出目錄
menu-settings = 設定...
menu-about = 關於 COSMIC 檔案總管...

## Sort

sort = 排序
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = 最新的在前
sort-oldest-first = 最舊的在前
sort-smallest-to-largest = 由小至大
sort-largest-to-smallest = 由大至小
deleted =
    已經刪除 { $items } { $items ->
        [one] 項目
       *[other] 項目
    } 從 { trash }
permanently-deleting =
    正在永久刪除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
permanently-deleted =
    已永久刪除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
removing-from-recents =
    正在從{ recents }中移除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
deleting =
    正在刪除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }從 { trash }（{ $progress }）…
removed-from-recents =
    已從{ recents }中移除 { $items } { $items ->
        [one] 项目
       *[other] 项目
    }
repository = 軟體庫源
desktop-view-options = 桌面檢視選項...
show-on-desktop = 顯示在桌面
desktop-folder-content = 桌面資料夾內容
mounted-drives = 已經掛載的磁碟機
trash-folder-icon = 垃圾桶圖示
trashed-on = 遺棄時間
icon-size-and-spacing = 圖示大小與間距
icon-size = 圖示大小
grid-spacing = 網格間距
details = 詳情
dismiss = 撤停訊息
delete = 刪除
remove = 移除
support = 支援
cancelled = 已取消
keywords = 資料夾;管理器;
empty-trash-title = 清空垃圾桶？
pause = 暫停
resume = 繼續
extract-password-required = 需要密碼
extract-to = 解壓縮至...
extract-to-title = 解壓縮至資料夾
mount-error = 無法存取磁碟機
open-with-title = 您要如何開啟「{ $name }」？
browse-store = 瀏覽 { $store }
other-apps = 其他應用程式
related-apps = 相關應用程式
permanently-delete-question = 永久刪除？
set-executable-and-launch = 設定為可以執行並啟動
read-only = 唯讀
read-execute = 讀取和執行
read-write = 讀取和寫入
read-write-execute = 讀取、寫入和執行
favorite-path-error = 開啟目錄時發生錯誤
set-executable-and-launch-description = 您是否要將「{ $name }」設為可執行並啟動它？
set-and-launch = 設定並啟動
none = 無
execute-only = 僅執行
write-only = 僅寫入
write-execute = 寫入和執行
operations-running =
    { $running } { $running ->
        [one] 個操作
       *[other] 個操作
    }正在執行（{ $percent }%）...
operations-running-finished =
    { $running } { $running ->
        [one] 個操作
       *[other] 個操作
    }正在執行（{ $percent }%）， { $finished } 個已經完成...
permanently-delete-warning = 「{ $target }」將被永久刪除。此操作無法復原。
open-with = 開啟檔案
selected-items = 已經選定 { $items } 個項目
copy-to-title = 選擇複製目的地
copy-to-button-label = 複製
move-to-title = 選擇移動目的地
move-to-button-label = 移動
keep = 保留
progress = { $percent }%
progress-cancelled = { $percent }%，已經取消
progress-failed = { $percent }%，失敗
progress-paused = { $percent }%，已經暫停
favorite-path-error-description =
    無法開啟「{ $path }」
    「{ $path }」可能不存在，或您可能沒有權限開啟它。

    您是否要將它從側邊欄移除？
comment = 檔案管理器 用於 COSMIC 桌面
pasted-image = 已經貼上的圖片
pasted-text = 已經貼上的文字
pasted-video = 已經貼上的影片
