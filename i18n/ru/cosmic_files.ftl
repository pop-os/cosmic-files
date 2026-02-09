cosmic-files = Файлы COSMIC
empty-folder = Папка пуста
empty-folder-hidden = Папка пуста (есть скрытые элементы)
no-results = Ничего не найдено
filesystem = Файловая система
home = Домашняя папка
trash = Корзина
networks = Сеть
notification-in-progress = Выполняются файловые операции
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
empty-trash-warning = Элементы в папке «Корзина» будут удалены без возможности восстановления
# New File/Folder Dialog
create-new-file = Создать новый файл
create-new-folder = Создать новую папку
file-name = Имя файла
folder-name = Имя папки
file-already-exists = Файл с таким именем уже существует.
folder-already-exists = Папка с таким именем уже существует.
name-hidden = Имена, начинающиеся на «.», будут скрыты.
name-invalid = Имя не может быть «{ $filename }».
name-no-slashes = Имя не должно содержать «/».
# Open/Save Dialog
cancel = Отмена
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
replace-title = { $filename } уже существует в данном каталоге.
replace-warning = Вы хотите заменить этот файл на тот, что сохраняете? Замена перезапишет все данные файла.
replace-warning-operation = Хотите заменить? Замена приведёт к перезаписи содержимого.
original-file = Оригинальный файл
replace-with = Заменить на
apply-to-all = Применить ко всем
keep-both = Сохранить оба
skip = Пропустить

## Metadata Dialog

owner = Владелец
group = Группа
other = Прочие

# Context Pages


## About


## Add Network Drive

add-network-drive = Добавить сетевой диск
connect = Подключиться
connect-anonymously = Подключиться анонимно
connecting = Подключение…
domain = Домен
enter-server-address = Введите адрес сервера
network-drive-description =
    Адреса серверов включают префикс протокола и адрес.
    Пример: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Доступные протоколы,Префикс
    AppleTalk,afp://
    File Transfer Protocol,ftp:// или ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// или ssh://
    WebDav,dav:// или davs://
network-drive-error = Невозможно получить доступ к сетевому диску
password = Пароль
remember-password = Запомнить пароль
try-again = Попробовать ещё раз
username = Имя пользователя

## Operations

edit-history = История редактирования
history = История
no-history = В истории нет записей.
pending = В процессе
failed = Не удалась
complete = Завершена
compressing =
    Сжатие { $items } { $items ->
        [one] элемента
       *[other] элем.
    } из «{ $from }» в «{ $to }» ({ $progress })…
compressed =
    Сжато { $items } { $items ->
        [one] элемент
       *[other] элем.
    } из «{ $from }» в «{ $to }»
copy_noun = Копирование
creating = Создание { $name } в { $parent }
created = Создан { $name } в { $parent }
copying =
    Копирование { $items } { $items ->
        [one] элемента
       *[other] элем.
    } из «{ $from }» в «{ $to }» ({ $progress })…
copied =
    Скопировано { $items } { $items ->
        [one] элемент
       *[other] элем.
    } из «{ $from }» в «{ $to }»
emptying-trash = Очистка { trash } ({ $progress })…
emptied-trash = { trash } очищена
extracting =
    Извлечение { $items } { $items ->
        [one] элемента
       *[other] элем.
    } из «{ $from }» в «{ $to }» ({ $progress })…
extracted =
    Извлечено { $items } { $items ->
        [one] элемент
       *[other] элем.
    } из «{ $from }» в «{ $to }»
moving =
    Перемещение { $items } { $items ->
        [one] элемента
       *[other] элем.
    } из «{ $from }» в «{ $to }» ({ $progress })…
moved =
    Перемещено { $items } { $items ->
        [one] элемент
       *[other] элем.
    } из «{ $from }» в «{ $to }»
renaming = Переименование «{ $from }» в «{ $to }»
renamed = «{ $from }» переименован в «{ $to }»
restoring =
    Восстановление { $items } { $items ->
        [one] элемента
       *[other] элем.
    } из { trash } ({ $progress })…
restored =
    Восстановлено { $items } { $items ->
        [one] элемент
       *[other] элем.
    } из { trash }
unknown-folder = неизвестная папка

## Open with

menu-open-with = Открыть с помощью…
default-app = { $name } (по умолчанию)

## Show details

show-details = Показать подробности

## Properties


## Settings

settings = Параметры

### Appearance

appearance = Оформление
theme = Тема
match-desktop = Как в системе
dark = Тёмная
light = Светлая
# Context menu
add-to-sidebar = Добавить на боковую панель
compress = Сжать
extract-here = Распаковать
new-file = Новый файл…
new-folder = Новая папка…
open-in-terminal = Открыть в терминале
move-to-trash = Переместить в корзину
restore-from-trash = Восстановить из корзины
remove-from-sidebar = Убрать с боковой панели
sort-by-name = Сорт. по имени
sort-by-modified = Сорт. по дате изменения
sort-by-size = Сорт. по размеру
sort-by-trashed = Сорт. по дате удаления

# Menu


## File

file = Файл
new-tab = Новая вкладка
new-window = Новое окно
rename = Переименовать…
close-tab = Закрыть вкладку
quit = Выйти

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
sort-a-z = От А до Я
sort-z-a = От Я до А
sort-newest-first = Сначала новые
sort-oldest-first = Сначала старые
sort-smallest-to-largest = От меньшего к большему
sort-largest-to-smallest = От большего к меньшему
support = Поддержка
repository = Репозиторий
cancelled = Отменена
details = Сведения
dismiss = Скрыть сообщение
remove = Убрать
desktop-view-options = Параметры вида рабочего стола…
show-on-desktop = Показывать на рабочем столе
desktop-folder-content = Содержимое папки рабочего стола
mounted-drives = Подключённые диски
trash-folder-icon = Значок папки корзины
icon-size-and-spacing = Размер и отступы значков
icon-size = Размер значка
grid-spacing = Отступ по сетке
pause = Приостановить
resume = Продолжить
extract-password-required = Требуется пароль
extract-to = Распаковать в…
extract-to-title = Распаковать в папку
mount-error = Не удалось получить доступ к диску
create = Создать
open-with-title = Как вы хотите открыть «{ $name }»?
browse-store = Искать в { $store }
other-apps = Другие приложения
related-apps = Связанные приложения
selected-items = { $items } выделенных элем.
permanently-delete-question = Навсегда удалить
delete = Удалить
permanently-delete-warning = Вы уверены, что хотите навсегда удалить { $target }? Это действие необратимо.
set-executable-and-launch = Сделать исполняемым и запустить
set-executable-and-launch-description = Вы хотите сделать «{ $name }» исполняемым и запустить его?
set-and-launch = Сделать и запустить
open-with = Открывать в
none = Нет прав
execute-only = Только исполнение
write-only = Только запись
write-execute = Запись и исполнение
read-only = Только чтение
read-execute = Чтение и исполнение
read-write = Чтение и запись
read-write-execute = Чтение, запись, исполнение
favorite-path-error = Не удалось открыть каталог
favorite-path-error-description =
    Не удалось открыть «{ $path }».
    Возможно, он не существует, либо у вас нет прав на его открытие.

    Хотите убрать его с боковой панели?
keep = Оставить
progress = { $percent } %
progress-cancelled = { $percent } %, отменена
progress-failed = { $percent } %, не удалась
progress-paused = { $percent } %, приостановлена
setting-executable-and-launching = Установка «{ $name }» исполняемым и запуск
set-executable-and-launched = «{ $name }» сделан исполнямым и запущен
setting-permissions = Изменение прав доступа «{ $name }» на { $mode }
set-permissions = Права доступа «{ $name }» изменены на { $mode }
type = Тип: { $mime }
items = Элементов: { $items }
item-size = Размер: { $size }
item-created = Дата создания: { $created }
item-modified = Дата изменения: { $modified }
item-accessed = Дата доступа: { $accessed }
calculating = Вычисление…
single-click = Открывать одним нажатием
type-to-search = Поле поиска
type-to-search-recursive = Поиск в текущей папке и подпапках
type-to-search-enter-path = Ввод пути к каталогу или файлу
delete-permanently = Удалить навсегда
eject = Извлечь
remove-from-recents = Убрать из недавних
change-wallpaper = Изменить обои…
desktop-appearance = Параметры оформления…
display-settings = Параметры экрана…
reload-folder = Обновить папку
gallery-preview = Галерея предпросмотра
operations-running =
    { $running } { $running ->
        [one] операция
       *[other] опер.
    } выполняется ({ $percent } %)…
operations-running-finished =
    { $running } { $running ->
        [one] операция
       *[other] опер.
    } выполняется ({ $percent } %), { $finished } завершено…
deleting =
    Удаление { $items } { $items ->
        [one] элемента
       *[other] элем.
    } из { trash } ({ $progress })…
deleted =
    Удалено { $items } { $items ->
        [one] элемент
       *[other] элем.
    } из { trash }
permanently-deleting =
    Удаление навсегда { $items } { $items ->
        [one] элемента
       *[other] элем.
    }
permanently-deleted =
    Удалено навсегда { $items } { $items ->
        [one] эелмент
       *[other] элем.
    }
removing-from-recents =
    Убирание { $items } { $items ->
        [one] элемента
       *[other] элем.
    } из { recents }
removed-from-recents =
    Убрано { $items } { $items ->
        [one] элемент
       *[other] элем.
    } из { recents }
type-to-search-select = Выделение первого подходящего файла или папки
pasted-image = Вставленное изображение
pasted-text = Вставленный текст
pasted-video = Вставленное видео
