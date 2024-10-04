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

# List view
name = Назва
modified = Зменена
size = Памер

# Dialogs

## Compress Dialog
create-archive = Стварыць архіў

## Empty Trash Dialog
empty-trash = Ачысціць сметніцу
empty-trash-warning = Вы сапраўды хочаце назаўсёды выдаліць усе элементы з сметніцы?

# New File/Folder Dialog
create-new-file = Стварыць новы файл
create-new-folder = Стварыць новую папку
file-name = Назва файла
folder-name = Назва папкі
file-already-exists = Файл з такой назвай ужо існуе.
folder-already-exists = Папка з такой назвай ужо існуе.
name-hidden = Назвы, якія пачынаюцца з ".", будуць схаваны.
name-invalid = Назва не можа быць "{$filename}".
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

# Rename Dialog
rename-file = Перайменаваць файл
rename-folder = Перайменаваць папку

# Replace Dialog
replace = Замяніць
replace-title = {$filename} ужо існуе ў гэтым месцы.
replace-warning = Вы сапраўды хочыце замяніць яго на той, які вы захоўваеце? Пры замене яго змесціва будзе перапісана.
replace-warning-operation = Вы хочаце замяніць яго? Пры замене яго змесціва будзе перазапісана.
original-file = Зыходны файл
replace-with = Замяніць на
apply-to-all = Прымяніць на ўсіх
keep-both = Захаваць абодва
skip = Прапусціць

## Metadata Dialog
owner = Уладальнік
group = Група
other = Іншыя
read = Чытаць
write = Пісаць
execute = Выконваць

# Context Pages

## About
git-description = Git каміт {$hash} ад {$date}

## Add Network Drive
add-network-drive = Дадаць сеткавы дыск
connect = Падлучыць
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
edit-history = Гісторыя рэдагавання
history = Гісторыя
no-history = Няма элементаў у гісторыі
pending = У чаканні
failed = Няўдала
complete = Завершана
compressing = Сцісканне {$items} {$items ->
        [one] элементу
        *[other] элементаў
    } з {$from} у {$to}
compressed = Сціснута {$items} {$items ->
        [one] элемент
        *[other] элементаў
    } з {$from} у {$to}
copy_noun = Капіяваць
creating = Стварэнне {$name} у {$parent}
created = Створана {$name} у {$parent}
copying = Капіяванне {$items} {$items ->
        [one] элементу
        *[other] элементаў
    } з {$from} у {$to}
copied = Скапіявана {$items} {$items ->
        [one] элемент
        *[other] элементаў
    } з {$from} у {$to}
emptying-trash = Emptying {trash}
emptied-trash = Emptied {trash}
extracting = Выманне {$items} {$items ->
        [one] элементу
        *[other] элементаў
    } з {$from} у {$to}
extracted = Вынята {$items} {$items ->
        [one] элемент
        *[other] элементаў
    } з {$from} у {$to}
moving = Перамяшчэнне {$items} {$items ->
        [one] элементу
        *[other] элементаў
    } з {$from} у {$to}
moved = Перанесена {$items} {$items ->
        [one] элемент
        *[other] элементаў
    } з {$from} у {$to}
renaming = Перайменаванне {$from} у {$to}
renamed = Перайменавана {$from} у {$to}
restoring = Аднаўленне {$items} {$items ->
        [one] элементу
        *[other] элементаў
    } з {trash}
restored = Адноўлена {$items} {$items ->
        [one] элемент
        *[other] элементаў
    } з {trash}
unknown-folder = невядомая папка

## Open with
open-with = Адкрыць з дапамогай
default-app = {$name} (па змаўчанні)

## Properties
properties = Уласцівасці

## Show details
show-details = Паказаць дэталі

## Settings
settings = Налады

### Appearance
appearance = Выгляд
theme = Тэма
match-desktop = Як у сістэме
dark = Цёмная
light = Светлая

# Context menu
add-to-sidebar = Дадаць на бакавую панэль
compress = Сціснуць
extract-here = Выняць
new-file = Новы файл
new-folder = Новая папка
open-in-terminal = Адкрыць у кансолі
move-to-trash = Перамясціць у сметніцу
restore-from-trash = Аднавіць са сметніцы
remove-from-sidebar = Выдаліць з бакавой панэлі
sort-by-name = Сартаваць па назве
sort-by-modified = Сартаваць па змяненні
sort-by-size = Сартаваць па памеры

# Menu

## File
file = Файл
new-tab = Новая ўкладка
new-window = Новае акно
rename = Перайменаваць
menu-show-details = Паказаць уласцівасці...
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
