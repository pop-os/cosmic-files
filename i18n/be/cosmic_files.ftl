cosmic-files = Файлы COSMIC
empty-folder = Пустая папка
empty-folder-hidden = Пустая папка (з захаванымі элементамі)
no-results = Нічога не знойдзена
filesystem = Файлавая сістэма
home = Хатняя папка
networks = Сеткі
notification-in-progress = Ідзе аперацыя з файламі.
trash = Сметніца
recents = Нядаўняе
undo = Адрабіць
today = Сёння
# Desktop view options
desktop-view-options = Параметры выгляду працоўнага стала...
show-on-desktop = Паказваць на працоўным стале
desktop-folder-content = Змесціва папкі "Працоўны стол"
mounted-drives = Змантаваныя дыскі
trash-folder-icon = Значок папкі "Сметніца"
icon-size-and-spacing = Памер і інтэрвал значкоў
icon-size = Памер значкоў
grid-spacing = Інтэрвал сеткі
# List view
name = Назва
modified = Зменена
trashed-on = У сметніцы
size = Памер
# Progress footer
details = Падрабязнасці
dismiss = Адхіліць паведамленне
operations-running =
    Выконваецца { $running } { $running ->
        [one] аперацыя
        [few] аперацыі
       *[other] аперацый
    } ({ $percent }%)...
operations-running-finished =
    Выконваецца { $running } { $running ->
        [one] аперацыя
        [few] аперацыі
       *[other] аперацый
    } ({ $percent }%), { $finished } завершана...
pause = Паўза
resume = Працягнуць

# Dialogs


## Compress Dialog

create-archive = Стварыць архіў

## Extract Dialog

extract-password-required = Патрабуецца пароль
extract-to = Выняць у...
extract-to-title = Выняць у папку

## Empty Trash Dialog

empty-trash = Ачысціць сметніцу
empty-trash-warning = Вы сапраўды хочаце назаўсёды выдаліць усе элементы з сметніцы?

## Mount Error Dialog

mount-error = Немагчыма атрымаць доступ да дыска
# New File/Folder Dialog
create-new-file = Стварыць новы файл
create-new-folder = Стварыць новую папку
file-name = Назва файла
folder-name = Назва папкі
file-already-exists = Файл з такой назвай ужо існуе.
folder-already-exists = Папка з такой назвай ужо існуе.
name-hidden = Назвы, якія пачынаюцца з ".", будуць схаваны.
name-invalid = Назва не можа быць "{ $filename }".
name-no-slashes = Назва не можа ўтрымліваць касыя рысы.
# Open/Save Dialog
cancel = Скасаваць
create = Стварыць
open = Адкрыць
open-file = Адкрыць файл
open-folder = Адкрыць папку
open-in-new-tab = Адкрыць у новай укладцы
open-in-new-window = Адкрыць у навым акне
open-item-location = Адкрыць месцазнаходжанне прадмета
open-multiple-files = Адкрыць некалькі файлаў
open-multiple-folders = Адкрыць некалькі папак
save = Захаваць
save-file = Захаваць файл

## Open With Dialog

open-with-title = Як вы хочаце адкрыць "{ $name }"?
browse-store = Прагляд { $store }
other-apps = Іншыя праграмы
related-apps = Звязаныя праграмы

## Permanently delete Dialog

selected-items = выбрана { $items } элементаў
permanently-delete-question = Выдалісь назаўжды
delete = Выдаліць
permanently-delete-warning = Вы ўпэўненыя, што хочаце назаўжды выдаліць { $target }? Гэта дзеянне немагчыма адмяніць.
# Rename Dialog
rename-file = Перайменаваць файл
rename-folder = Перайменаваць папку
# Replace Dialog
replace = Замяніць
replace-title = { $filename } ужо існуе ў гэтым месцы.
replace-warning = Вы сапраўды хочыце замяніць яго на той, які вы захоўваеце? Пры замене яго змесціва будзе перапісана.
replace-warning-operation = Вы хочаце замяніць яго? Пры замене яго змесціва будзе перазапісана.
original-file = Зыходны файл
replace-with = Замяніць на
apply-to-all = Прымяніць на ўсіх
keep-both = Захаваць абодва
skip = Прапусціць

## Set as Executable and Launch Dialog

set-executable-and-launch = Зрабіць выконвальным і запусціць
set-executable-and-launch-description = Вы хочаце зрабіць "{ $name }" выконвальным і запусціць?
set-and-launch = Задаць і запусціць

## Metadata Dialog

open-with = Адкрыць праз
owner = Уладальнік
group = Група
other = Іншыя

### Mode 0

none = Няма

### Mode 1 (unusual)

execute-only = Толькі выкананне

### Mode 2 (unusual)

write-only = Толькі запіс

### Mode 3 (unusual)

write-execute = Запіс і выкананне

### Mode 4

read-only = Толькі чытанне

### Mode 5

read-execute = Чытанне і выкананне

### Mode 6

read-write = Чытанне і запіс

### Mode 7

read-write-execute = Чытанне, запіс і выкананне

## Favorite Path Error Dialog

favorite-path-error = Памылка адкрыцця каталога
favorite-path-error-description =
    Немагчыма адкрыць "{ $path }".
    Магчыма, ён не існуе ці ў вас няма дазволу на яго адкрыццё.

    Ці жадаеце вы выдаліць яго з бакавой панэлі?
remove = Выдаліць
keep = Захаваць

# Context Pages


## About

git-description = Git каміт { $hash } ад { $date }

## Add Network Drive

add-network-drive = Дадаць сеткавы дыск
connect = Падключыцца
connect-anonymously = Падлучыць ананімна
connecting = Падлучэнне...
domain = Дамен
enter-server-address = Увядзіце адрас серверу
network-drive-description =
    Адрасы сервераў ўключаюць у сябе прэфікс пратаколу і адрас.
    Прыклад: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Даступная пратаколы,Прэфікс
    AppleTalk,afp://
    File Transfer Protocol,ftp:// або ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// або ssh://
    WebDav,dav:// або davs://
network-drive-error = Немагчыма атрымаць доступ да сеткавага дыска
password = Пароль
remember-password = Запомніць пароль
try-again = Паўтарыць спробу
username = Імя карыстальніка

## Operations

cancelled = Скасавана
edit-history = Гісторыя рэдагавання
history = Гісторыя
no-history = У гісторыі няма запісаў.
pending = У чаканні
progress = { $percent }%
progress-cancelled = { $percent }%, скасавана
progress-paused = { $percent }%, прыпынена
failed = Няўдала
complete = Завершана
compressing =
    Сцісканне { $items } { $items ->
        [one] элемента
        [few] элементаў
       *[other] элементаў
    } з «{ $from }» у «{ $to }» ({ $progress })...
compressed =
    Сціснута { $items } { $items ->
        [one] элемент
        [few] элементы
       *[other] элементаў
    } з { $from } у { $to }
copy_noun = Капіяваць
creating = Стварэнне { $name } у { $parent }
created = Створана { $name } у { $parent }
copying =
    Капіяванне { $items } { $items ->
        [one] элемента
        [few] элементаў
       *[other] элементаў
    } з «{ $from }» у «{ $to }» ({ $progress })...
copied =
    Скапіявана { $items } { $items ->
        [one] элемент
        [few] элементы
       *[other] элементаў
    } з { $from } у { $to }
deleting =
    Выдаленне { $items } { $items ->
        [one] элемента
        [few] элементы
       *[other] элементаў
    } са { trash } ({ $progress })...
deleted =
    Выдалена { $items } { $items ->
        [one] элемент
        [few] элементы
       *[other] элементаў
    } са { trash }
emptying-trash = Ачыстка { trash } ({ $progress })…
emptied-trash = Ачышчана { trash }
extracting =
    Выманне { $items } { $items ->
        [one] элемента
        [few] элементаў
       *[other] элементаў
    } з «{ $from }» у «{ $to }» ({ $progress })...
extracted =
    Вынята { $items } { $items ->
        [one] элемент
        [few] элементы
       *[other] элементаў
    } з { $from } у { $to }
setting-executable-and-launching = Робім "{ $name }" выконвальным і запускаем
set-executable-and-launched = "{ $name }" зроблены выконвальным і запушчаны
setting-permissions = Усталёўваем дазволы для "{ $name }" на { $mode }
set-permissions = Дазволы для "{ $name }" усталяваны на { $mode }
moving =
    Перамяшчэнне { $items } { $items ->
        [one] элемента
        [few] элементаў
       *[other] элементаў
    } з «{ $from }» у «{ $to }» ({ $progress })...
moved =
    Перанесена { $items } { $items ->
        [one] элемент
        [few] элементы
       *[other] элементаў
    } з { $from } у { $to }
permanently-deleting =
    Назаўсёды выдаляем { $items } { $items ->
        [one] элемент
        [few] элементы
       *[other] элементаў
    }
permanently-deleted =
    Назаўсёды выдалена { $items } { $items ->
        [one] элемент
        [few] элементы
       *[other] элементаў
    }
renaming = Перайменаванне { $from } у { $to }
renamed = Перайменавана { $from } у { $to }
restoring =
    Аднаўленне { $items } { $items ->
        [one] элемента
        [few] элементаў
       *[other] элементаў
    } з { trash } ({ $progress })...
restored =
    Адноўлена { $items } { $items ->
        [one] элемент
        [few] элементы
       *[other] элементаў
    } з { trash }
unknown-folder = невядомая папка

## Open with

menu-open-with = Адкрыць праз...
default-app = { $name } (па змаўчанні)

## Show details

show-details = Паказаць дэталі
type = Тып: { $mime }
items = Элементаў: { $items }
item-size = Памер: { $size }
item-created = Створана: { $created }
item-modified = Зменена: { $modified }
item-accessed = Апошні доступ: { $accessed }
calculating = Вылічэнне...

## Settings

settings = Налады
single-click = Адзін клік каб адкрыць

### Appearance

appearance = Выгляд
theme = Тэма
match-desktop = Як у сістэме
dark = Цёмная
light = Светлая

### Type to Search

type-to-search = Увядзіце для пошуку
type-to-search-recursive = Шукае ў бягучай папцы і ва ўсіх укладзеных папках
type-to-search-enter-path = Уводзіць шлях да каталога або файла
# Context menu
add-to-sidebar = Дадаць на бакавую панэль
compress = Сціснуць
delete-permanently = Выдаліць назаўжды
eject = Выняць
extract-here = Выняць
new-file = Новы файл...
new-folder = Новая папка...
open-in-terminal = Адкрыць у кансолі
move-to-trash = Перамясціць у сметніцу
restore-from-trash = Аднавіць са сметніцы
remove-from-sidebar = Выдаліць з бакавой панэлі
sort-by-name = Сартаваць па назве
sort-by-modified = Сартаваць па змяненні
sort-by-size = Сартаваць па памеры
sort-by-trashed = Сартаваць па часе выдалення

## Desktop

change-wallpaper = Змяніць шпалеры...
desktop-appearance = Выгляд працоўнага стала...
display-settings = Налады дысплэя...

# Menu


## File

file = Файл
new-tab = Новая ўкладка
new-window = Новае акно
reload-folder = Аднавіць папку
rename = Перайменаваць...
close-tab = Закрыць укладку
quit = Выйсці

## Edit

edit = Рэдагаваць
cut = Выразаць
copy = Скапіяваць
paste = Уставіць
select-all = Вылучыць усё

## View

zoom-in = Павялічыць
default-size = Памер па змаўчанні
zoom-out = Паменшыць
view = Выгляд
grid-view = Рэжым сеткі
list-view = Рэжым спіса
show-hidden-files = Паказваць схаваныя файлы
list-directories-first = Размяшчаць папкі перад файламі
gallery-preview = Папярэдні прагляд
menu-settings = Налады...
menu-about = Пра Файлы COSMIC...

## Sort

sort = Сартаванне
sort-a-z = А-Я
sort-z-a = Я-А
sort-newest-first = Спачатку новыя
sort-oldest-first = Спачатку старыя
sort-smallest-to-largest = Ад меншага да найбольшага
sort-largest-to-smallest = Ад вялікага да найменшага
repository = Рэпазіторый
support = Падтрымка
