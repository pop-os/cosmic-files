cosmic-files = Файлы COSMIC
empty-folder = Пустая папка
empty-folder-hidden = Пустая папка (со скрытыми элементами)
no-results = Ничего не найдено
filesystem = Файловая система
home = Домашняя папка
trash = Корзина
networks = Сеть
notification-in-progress = Выполняются файловые операции.
recents = Недавние документы
undo = Отменить
today = Сегодня

# List view
name = Имя
modified = Изменено
trashed-on = Удалено
size = Размер

# Dialogs

## Compress Dialog
create-archive = Создать архив

## Empty Trash Dialog
empty-trash = Очистить корзину
empty-trash-warning = Вы уверены, что хотите навсегда удалить все элементы в корзине?

# New File/Folder Dialog
create-new-file = Создать новый файл
create-new-folder = Создать новую папку
file-name = Имя файла
folder-name = Имя папки
file-already-exists = Файл с таким именем уже существует.
folder-already-exists = Папка с таким именем уже существует.
name-hidden = Имена начинающиеся на "." будут скрыты.
name-invalid = Невозможно дать имя "{$filename}".
name-no-slashes = Имя не должно содержать "/".

# Open/Save Dialog
cancel = Отменить
open = Открыть
open-file = Открыть файл
open-folder = Открыть папку
open-in-new-tab = Открыть в новой вкладке
open-in-new-window = Открыть в новом окне
open-item-location = Открыть расположение элемента
open-multiple-files = Открыть несколько файлов
open-multiple-folders = Открыть несколько папок
save = Сохранить
save-file = Сохранить файл

# Rename Dialog
rename-file = Переименовать файл
rename-folder = Переименовать папку

# Replace Dialog
replace = Заменить
replace-title = {$filename} уже существует в данном каталоге.
replace-warning = Действительно ли Вы хотите заменить файл на тот, что Вы сохраняете? Замена перезапишет все данные файла.
replace-warning-operation = Хотите заменить? Замена приведёт к перезаписи содержимого.
original-file = Оригинальный файл
replace-with = Заменить на
apply-to-all = Применить ко всем
keep-both = Сохранить оба
skip = Пропустить

## Metadata Dialog
owner = Владелец
group = Группа
other = Остальные
read = Чтение
write = Запись
execute = Выполнение

# Context Pages

## About
git-description = Git коммит {$hash} от {$date}

## Add Network Drive
add-network-drive = Добавить сетевой диск
connect = Подключиться
connect-anonymously = Подключиться анонимно
connecting = Подключение...
domain = Домен
enter-server-address = Введите адрес сервера
network-drive-description =
    Адреса серверов включают префикс протокола и адрес.
    Пример: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Доступные протоколы,Префикс
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Невозможно получить доступ к сетевому диску
password = Пароль
remember-password = Запомнить пароль
try-again = Попробовать еще раз
username = Имя пользователя

## Operations
operations = Операции
edit-history = История редактирования
history = История
no-history = В истории нет записей.
pending = В ожидании
failed = Сбой
complete = Завершено
compressing = Сжатие {$items} {$items ->
        [one] элемента
        *[other] элементов
    } из {$from} в {$to}
compressed = Сжат {$items} {$items ->
        [one] элемент
        *[other] элементы
    } из {$from} в {$to}
copy_noun = Копирование
creating = Создание {$name} в {$parent}
created = Создан {$name} в {$parent}
copying = Копирование {$items} {$items ->
        [one] элемент
        *[other] элементов
    } из {$from} в {$to}
copied = Скопировано {$items} {$items ->
        [one] элемент
        *[other] элементов
    } из {$from} в {$to}
emptying-trash = Очистка {trash}
emptied-trash = Очищена {trash}
extracting = Извлечение {$items} {$items ->
        [one] элемента
        *[other] элементов
    } из {$from} в {$to}
extracted = Извлечено {$items} {$items ->
        [one] элемент
        *[other] элементы
    } из {$from} в {$to}
moving = Перемещение {$items} {$items ->
        [one] элемента
        *[other] элементов
    } из {$from} в {$to}
moved = Перемещено {$items} {$items ->
        [one] элемент
        *[other] элементы
    } из {$from} в {$to}
renaming = Переименование {$from} в {$to}
renamed = Переименован {$from} в {$to}
restoring = Восстановление {$items} {$items ->
        [one] элемента
        *[other] элементов
    } из {trash}
restored = Восстановлено {$items} {$items ->
        [one] элемент
        *[other] элементов
    } из {trash}
unknown-folder = неизвестная папка

## Open with
open-with = Открыть с помощью
default-app = {$name} (по умолчанию)

## Show details
show-details = Показать подробности

## Properties
properties = Свойства

## Settings
settings = Параметры

### Appearance
appearance = Оформление
theme = Тема
match-desktop = Как в системе
dark = Темная
light = Светлая

# Context menu
add-to-sidebar = Добавить на боковую панель
compress = Сжать
extract-here = Извлечь
new-file = Новый файл
new-folder = Новая папка
open-in-terminal = Открыть в терминале
move-to-trash = Переместить в корзину
restore-from-trash = Восстановить из корзины
remove-from-sidebar = Убрать из боковой панели
sort-by-name = Разместить по имени
sort-by-modified = Разместить по дате изменения
sort-by-size = Разместить по размеру
sort-by-trashed = Разместить по дате удаления

# Menu

## File
file = Файл
new-tab = Новая вкладка
new-window = Новое окно
rename = Переименовать
menu-show-details = Показать подробности
close-tab = Закрыть вкладку
quit = Завершить

## Edit
edit = Правка
cut = Вырезать
copy = Копировать
paste = Вставить
select-all = Выбрать все

## View
zoom-in = Увеличить
default-size = Размер по умолчанию
zoom-out = Уменьшить
view = Вид
grid-view = Сетка
list-view = Список
show-hidden-files = Показывать скрытые файлы
list-directories-first = Показывать сначала папки
menu-settings = Параметры...
menu-about = О приложении Файлы COSMIC...

## Sort
sort = Сортировка
sort-a-z = По алфавиту (от А до Я)
sort-z-a = По алфавиту (от Я до А)
sort-newest-first = Сначала новые
sort-oldest-first = Сначала старые
sort-smallest-to-largest = От наименьшего к большему
sort-largest-to-smallest = От большего к меньшему