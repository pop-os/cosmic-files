cosmic-files = COSMIC 檔案總管
empty-folder = 空資料夾
empty-folder-hidden = 空資料夾（包含隱藏項目）
no-results = 找不到結果
filesystem = 檔案系統
home = 主目錄
networks = 網路
notification-in-progress = 檔案操作正在進行中。
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
empty-trash-warning = 你確定要永久刪除垃圾桶中的所有項目嗎？

## New File/Folder Dialog
create-new-file = 建立新檔案
create-new-folder = 建立新資料夾
file-name = 檔案名稱
folder-name = 資料夾名稱
file-already-exists = 已存在同名檔案。
folder-already-exists = 已存在同名資料夾。
name-hidden = 以「.」開頭的名稱將會被隱藏。
name-invalid = 名稱不能是 "{$filename}"。
name-no-slashes = 名稱不能包含斜線。

## Open/Save Dialog
cancel = 取消
create = 建立
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
replace-title = 檔案 {$filename} 已存在於此位置。
replace-warning = 你要取代它嗎？取代將覆蓋其內容。
replace-warning-operation = 你要取代它嗎？取代將覆蓋其內容。
original-file = 原始檔案
replace-with = 取代為
apply-to-all = 套用至所有項目
keep-both = 保留兩者
skip = 跳過

## Metadata Dialog
owner = 擁有者
group = 群組
other = 其他
read = 讀取
write = 寫入
execute = 執行

# Context Pages

## About
git-description = Git 提交 {$hash} 於 {$date}

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
no-history = 沒有歷史項目。
pending = 待處理
failed = 失敗
complete = 完成
compressing = 正在壓縮 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從 {$from} 到 {$to}
compressed = 已壓縮 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從 {$from} 到 {$to}
copy_noun = 複製
creating = 正在建立 {$name} 於 {$parent}
created = 已建立 {$name} 於 {$parent}
copying = 正在複製 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從 {$from} 到 {$to}
copied = 已複製 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從 {$from} 到 {$to}
emptying-trash = 正在清空垃圾桶
emptied-trash = 已清空垃圾桶
extracting = 正在解壓縮 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從 {$from} 到 {$to}
extracted = 已解壓縮 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從 {$from} 到 {$to}
moving = 正在移動 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從 {$from} 到 {$to}
moved = 已移動 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從 {$from} 到 {$to}
renaming = 正在重新命名 {$from} 為 {$to}
renamed = 已重新命名 {$from} 為 {$to}
restoring = 正在還原 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從垃圾桶
restored = 已還原 {$items} 項目 {$items ->
        [one] 項目
        *[other] 項目
    } 從垃圾桶
unknown-folder = 未知資料夾

## Open with
open-with = 開啟方式...
default-app = {$name} （預設）

## Show details
show-details = 顯示詳細資料

## Settings
settings = 設定

### Appearance
appearance = 外觀
theme = 主題
match-desktop = 與桌面一致
dark = 暗色模式
light = 亮色模式

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
new-tab = 新分頁
new-window = 新視窗
rename = 重新命名...
menu-show-details = 顯示詳細資料...
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