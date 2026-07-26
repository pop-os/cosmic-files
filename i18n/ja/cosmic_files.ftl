cosmic-files = COSMICファイル
empty-folder = 空のフォルダ
empty-folder-hidden = 空のフォルダ（隠しファイルあり）
no-results = 検索結果はありません
filesystem = ファイルシステム
home = ホーム
networks = ネットワーク
notification-in-progress = ファイル処理が進行中です。
trash = ゴミ箱
recents = 最近
undo = 元に戻す
# List view
name = 名前
modified = 更新日
size = サイズ

# Dialogs


## Compress Dialog

create-archive = アーカイブを作成

## Empty Trash Dialog

empty-trash = ゴミ箱を空にする
empty-trash-warning = ゴミ箱のアイテムをすべて完全に削除してもよろしいですか？
# New File/Folder Dialog
create-new-file = 新しいファイルを作成
create-new-folder = 新しいフォルダを作成
file-name = ファイル名
folder-name = フォルダ名
file-already-exists = 同じ名前のファイルがすでに存在します。
folder-already-exists = 同じ名前のフォルダがすでに存在します。
name-hidden = 「.」で始まる名前は隠られます。
name-invalid = 「{ $filename }」という名前は使用できません。
name-no-slashes = 「/」は名前に含められません。
# Open/Save Dialog
cancel = キャンセル
create = 作る
open = 開く
open-file = ファイルを開く
open-folder = フォルダを開く
open-in-new-tab = 新しいタブで開く
open-in-new-window = 新しいウィンドウで開く
open-item-location = アイテムの場所を開く
open-multiple-files = 複数ファイルを開く
open-multiple-folders = 複数フォルダを開く
save = 保存
save-file = ファイルを保存
# Rename Dialog
rename-file = ファイル名を変更
rename-folder = フォルダ名を変更
# Replace Dialog
replace = 置き換える
replace-title = { $filename }はすでにこの場所に存在します。
replace-warning = 保存しているファイルで置き換えますか？置き換えると、内容を上書きます。
replace-warning-operation = 置き換えますか？置き換えると、内容を上書きます。
original-file = 元のファイル
replace-with = これで置き換える：
apply-to-all = 全てに適用
keep-both = 両方を保管
skip = スキップ

## Metadata Dialog

owner = 所有者
group = グループ
other = その他

# Context Pages


## About


## Add Network Drive

add-network-drive = ネットワークドライブを追加
connect = 接続する
connect-anonymously = 匿名的に接続
connecting = 接続中...
domain = ドメイン
enter-server-address = サーバーアドレスを入力
network-drive-description =
    サーバーアドレスはプロトコル接頭辞とアドレスを含めます。
    例: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    利用可能なプロトコル,接頭辞
    AppleTalk,afp://
    File Transfer Protocol,ftp:// または ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// または ssh://
    WebDav,dav:// または davs://
network-drive-error = ネットワークドライブにアクセスできませんでした
password = パスワード
remember-password = パスワードを覚える
try-again = 再試行
username = ユーザー名

## Operations

edit-history = 履歴を編集
history = 履歴
no-history = 履歴はありません。
pending = 保留中
failed = 失敗
complete = 完了
compressing =
    "{ $from }" から "{ $to }" へ { $items }個のアイテムを圧縮中 ({ $progress }) { $items ->
        [one] 項目
       *[other] 項目
    }…
compressed =
    "{ $from }" から "{ $to }" へ { $items }個のアイテムを圧縮しました { $items ->
        [one] 項目
       *[other] 項目
    }
copy_noun = コピー
creating = { $parent }で{ $name }を作成中
created = { $parent }で{ $name }を作成完了
copying =
    "{ $from }" から "{ $to }" へ { $items }個のアイテムをコピー中 ({ $progress }) { $items ->
        [one] 項目
       *[other] 項目
    }…
copied =
    "{ $from }" から "{ $to }" へ { $items }個のアイテムをコピーしました { $items ->
        [one] 項目
       *[other] 項目
    }
emptying-trash = { trash }を空にしています ({ $progress })…
emptied-trash = { trash }を空にした
moving =
    "{ $from }" から "{ $to }" へ { $items }個のアイテムを移動中 ({ $progress }) { $items ->
        [one] 項目
       *[other] 項目
    }…
moved =
    "{ $from }" から "{ $to }" へ { $items }個のアイテムを移動しました { $items ->
        [one] 項目
       *[other] 項目
    }
renaming = { $from }を{ $to }に変更中
renamed = { $from }を{ $to }に変更完了
restoring =
    { trash }から{ $items }個のアイテムを復元中 ({ $progress }) { $items ->
        [one] 項目
       *[other] 項目
    }…
restored =
    { trash }から{ $items }個のアイテムを復元しました { $items ->
        [one] 項目
       *[other] 項目
    }
unknown-folder = 不明なフォルダー

## Open with

menu-open-with = 別のアプリケーションで開く...
default-app = { $name } (デフォルト)

## Properties


## Settings

settings = 設定

### Appearance

appearance = 外観
theme = テーマ
match-desktop = システム設定に従う
dark = ダーク
light = ライト
# Context menu
add-to-sidebar = サイドバーに追加
compress = 圧縮
extract-here = 抽出
new-file = 新しいファイル...
new-folder = 新しいフォルダ...
open-in-terminal = 端末で開く
move-to-trash = ゴミ箱に移動
restore-from-trash = ゴミ箱から復元
remove-from-sidebar = サイドバーから削除
sort-by-name = 名前で並べ替え
sort-by-modified = 更新日で並べ替え
sort-by-size = サイズで並べ替え

# Menu


## File

file = ファイル
new-tab = 新しいタブ
new-window = 新しいウィンドウ
rename = 名前を変更...
close-tab = タブを閉じる
quit = 終了

## Edit

edit = 編集
cut = 切り取り
copy = コピー
paste = 貼り付け
select-all = すべてを選択

## View

zoom-in = ズームイン
default-size = 規定のサイズ
zoom-out = ズームアウト
view = 表示
grid-view = グリッドの表示
list-view = リストの表示
show-hidden-files = 隠しファイルを表示
list-directories-first = フォルダを最初に表示
menu-settings = 設定...
menu-about = COSMICファイルについて...

## Sort

sort = 並べ替え
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = 新しい順
sort-oldest-first = 古い順
sort-smallest-to-largest = 最小から最大まで
sort-largest-to-smallest = 最大から最小まで
repository = リポジトリ
support = サポート
remove = 削除
today = 今日
desktop-view-options = デスクトップの表示オプション…
show-on-desktop = デスクトップの表示オプション
desktop-folder-content = デスクトップフォルダの内容
mounted-drives = マウント済みドライブ
trash-folder-icon = ゴミ箱のアイコン
icon-size-and-spacing = アイコンのサイズと間隔
icon-size = アイコンサイズ
grid-spacing = グリッドの間隔
trashed-on = ゴミ箱に入れた日時
details = 詳細
dismiss = メッセージを閉じる
operations-running =
    { $running } { $running ->
        [one] 件の操作が
       *[other] 件の操作が
    }実行中です({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] 件の操作が
       *[other] 件の操作が
    } 実行中です({ $percent }%)、 { $finished } 件が終了...
pause = 一時停止
resume = 一時停止
extract-password-required = パスワードが必要です
extract-to = 展開先…
extract-to-title = フォルダーに展開
mount-error = ドライブにアクセスできません
open-with-title = 「{ $name }」をどのように開きますか？
browse-store = { $store } を参照
other-apps = 他のアプリケーション
related-apps = 関連アプリケーション
selected-items = 選択された{ $items }個のアイテム
permanently-delete-question = 完全に削除
delete = 削除
permanently-delete-warning = { $target }を完全に削除してもよろしいですか？この操作は元に戻せません。
set-executable-and-launch = 実行可能にして起動
set-executable-and-launch-description = "{ $name }"を実行可能に設定して起動しますか？
set-and-launch = 設定して起動
open-with = 別のアプリケーションで開く
none = なし
execute-only = 実行のみ
write-only = 書き込み専用
write-execute = 書き込みと実行
read-only = 読み取り専用
read-execute = 読み取りと実行
read-write = 読み取りと書き込み
read-write-execute = 読み取り、書き込み、実行
favorite-path-error = ディレクトリを開けませんでした
favorite-path-error-description =
    "{ $path }"を開けません。
    このパスが存在しないか、開くための権限がない可能性があります。

    サイドバーから削除しますか？
keep = そのままにする
cancelled = キャンセルされました
progress = { $percent } %
progress-cancelled = { $percent } %、キャンセルされました
progress-failed = { $percent } %、失敗
progress-paused = { $percent } %、一時停止中
deleting =
    { $items }個のアイテムを{ trash }から削除中 ({ $progress }) { $items ->
        [one] 項目
       *[other] 項目
    }…
deleted =
    { $items } 個のアイテムを { trash } から削除しました { $items ->
        [one] 項目
       *[other] 項目
    }
extracting =
    "{ $from }" から "{ $to }" へ { $items } 個のアイテムを展開中 ({ $progress }) { $items ->
        [one] 項目
       *[other] 項目
    }…
extracted =
    "{ $from }" から "{ $to }" へ { $items } 個のアイテムを展開しました { $items ->
        [one] 項目
       *[other] 項目
    }
setting-executable-and-launching = "{ $name }"を実行可能に設定して起動中
set-executable-and-launched = "{ $name }"を実行可能に設定して起動しました
setting-permissions = "{ $name }"のパーミッションを{ $mode }に設定中
set-permissions = "{ $name }"のパーミッションを{ $mode }に設定しました
permanently-deleting =
    { $items }個のアイテムを完全に削除中 { $items ->
        [one] 項目
       *[other] 項目
    }
permanently-deleted =
    { $items }個のアイテムを完全に削除しました { $items ->
        [one] 項目
       *[other] 項目
    }
removing-from-recents =
    { $items }個のアイテムを{ recents }から削除中 { $items ->
        [one] 項目
       *[other] 項目
    }
removed-from-recents =
    { $items }個のアイテムを{ recents }から削除しました { $items ->
        [one] 項目
       *[other] 項目
    }
show-details = 詳細を表示
type = 種類: { $mime }
items = アイテム: { $items }
item-size = サイズ: { $size }
item-created = 作成日時: { $created }
item-modified = 最終更新日時: { $modified }
item-accessed = 最終アクセス日時: { $accessed }
calculating = 計算中…
single-click = シングルクリックで開く
type-to-search = 入力して検索
type-to-search-recursive = 現在のフォルダーとすべてのサブフォルダーを検索
type-to-search-enter-path = ディレクトリーまたはファイルのパスを入力
delete-permanently = 完全に削除する
eject = 取り出し
sort-by-trashed = 削除日時
remove-from-recents = 最近の項目から削除
change-wallpaper = 壁紙を変更…
desktop-appearance = デスクトップの見た目…
display-settings = ディスプレイの設定…
reload-folder = フォルダーを再読み込み
gallery-preview = ギャラリープレビュー
