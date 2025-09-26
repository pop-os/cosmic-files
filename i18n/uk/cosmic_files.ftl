cosmic-files = Файли COSMIC
empty-folder = Тека порожня
empty-folder-hidden = Тека порожня (але містить приховані елементи)
filesystem = Файлова система
home = Домівка
trash = Смітник
recents = недавній
undo = Скасувати
# List view
name = Назва
modified = Змінено
size = Розмір

# Dialogs


## Empty Trash Dialog

empty-trash = Спорожнити смітник
empty-trash-warning = Ви впевнені, що хочете вилучити назавжди всі обʼєкти зі смітника?

## New File/Folder Dialog

create-new-file = Створити новий файл
create-new-folder = Створити нову теку
file-name = Назву файлу
folder-name = Назва теки
file-already-exists = Файл з такою назвою вже існує.
folder-already-exists = Тека з такою назвою вже існує.
name-hidden = Назви, що починаються з ".", будуть приховані.
name-invalid = Назва "{ $filename }" є неприйнятною.
name-no-slashes = Назва не може містити слеш.

## Open/Save Dialog

cancel = Скасувати
open = Відкрити
open-file = Відкрити файл
open-folder = Відкрити теку
open-in-new-tab = Відкрити в новій вкладці
open-in-new-window = Відкрити в новому вікні
open-multiple-files = Відкрити кілька файлів
open-multiple-folders = Відкрити кілька тек
save = Зберегти
save-file = Зберегти файл

## Rename Dialog

rename-file = Перейменувати файл
rename-folder = Перейменувати теку

## Replace Dialog

replace = Замінити
replace-title = Файл із назвою { $filename } вже існує.
replace-warning = Замінити його на той, що ви зберігаєте зараз? Заміна призведе до перезапису його вмісту.
replace-warning-operation = Ви хочете замінити його? Заміна призведе до перезапису його вмісту.
original-file = Початковий файл
replace-with = Замінити на
apply-to-all = Застосувати до всіх
keep-both = Залишити обидва
skip = Пропустити

# Context Pages


## About

git-description = Git commit { $hash } за { $date }

## Operations

edit-history = Редагувати історію
history = Історія
no-history = Відсутні обʼєкти в історії.
pending = Очікують
failed = Невдалі
complete = Завершені
copy_noun = Копіювати
creating = Створення { $name } в { $parent }
created = Створено { $name } в { $parent }
copying =
    Копіювання { $items } { $items ->
        [one] обʼєкта
       *[other] обʼєктів
    } з { $from } до "{ $to }" ({ $progress })...
copied =
    Скопійовано { $items } { $items ->
        [one] обʼєкт
        [few] обʼєкти
        [many] об'єктів
       *[other] обʼєктів
    } з { $from } до { $to }
emptying-trash = Спорожнення { trash } ({ $progress })...
emptied-trash = Спорожнено { trash }
moving =
    Переміщення { $items } { $items ->
        [one] обʼєкта
       *[other] обʼєктів
    } з { $from } до "{ $to }" ({ $progress })...
moved =
    Переміщено { $items } { $items ->
        [one] обʼєкт
        [few] обʼєкти
        [many] об'єктів
       *[other] обʼєктів
    } з "{ $from }" до "{ $to }"
renaming = Перейменування { $from } на { $to }
renamed = Перейменовано { $from } на { $to }
restoring =
    Відновлення { $items } { $items ->
        [one] обʼєкта
       *[other] обʼєктів
    } зі { trash } ({ $progress })...
restored =
    Відновлено { $items } { $items ->
        [one] обʼєкт
        [few] обʼєкти
        [many] об'єктів
       *[other] обʼєктів
    } зі { trash }
unknown-folder = невідома тека

## Open with

menu-open-with = Відкрити за допомогою...
default-app = { $name } (типово)

## Properties

properties = Властивості

## Settings

settings = Налаштування

### Appearance

appearance = Зовнішній вигляд
theme = Тема
match-desktop = Системна
dark = Темна
light = Світла
# Context menu
add-to-sidebar = Додати до бічної панелі
new-file = Новий файл...
new-folder = Нова тека...
open-in-terminal = Відкрити у терміналі
move-to-trash = Перемістити до смітника
restore-from-trash = Відновити зі смітника
remove-from-sidebar = Вилучити з бічної панелі
sort-by-name = Сортувати за назвою
sort-by-modified = Сортувати за зміною
sort-by-size = Сортувати за розміром

# Menu


## File

file = Файл
new-tab = Нова вкладка
new-window = Нове вікно
rename = Перейменувати...
close-tab = Закрити вкладку
quit = Вийти

## Edit

edit = Зміни
cut = Вирізати
copy = Копіювати
paste = Вставити
select-all = Вибрати все

## View

zoom-in = Збільшити шрифт
default-size = Стандартний розмір
zoom-out = Зменшити шрифт
view = Перегляд
grid-view = Перегляд ґраткою
list-view = Перегляд списком
show-hidden-files = Показувати приховані файли
list-directories-first = Спершу показувати теки
menu-settings = Налаштування...
menu-about = Про Файли COSMIC...
repository = Репозиторій
support = Підтримка
details = Деталі
dismiss = Сховати повідомлення
remove = Видалити
cancelled = Скасовані
no-results = Нічого не знайдено
networks = Мережа
notification-in-progress = Виконуються операції з файлами.
today = Сьогодні
desktop-view-options = Параметри іконок стільниці...
show-on-desktop = Показувати на стільниці
desktop-folder-content = Вміст теки Стільниця
mounted-drives = Змонтовані диски
trash-folder-icon = Іконку теки Смітник
icon-size-and-spacing = Розмір іконок та відстань між ними
icon-size = Розмір іконок
grid-spacing = Відстань між іконками
trashed-on = В смітнику
operations-running =
    { $running } { $running ->
        [zero] операцій
        [one] операція
        [few] операції
        [many] операцій
       *[other] операцій
    } запущено ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [zero] операцій
        [one] операція
        [few] операції
        [many] операцій
       *[other] операцій
    } запущено ({ $percent }%), з них { $finished } виконано...
pause = Пауза
resume = Відновити
create-archive = Створити архів
extract-password-required = Потрібен пароль
extract-to = Видобути до...
extract-to-title = Видобути до теки
mount-error = Неможливо отримати доступ до диску
create = Створити
open-item-location = Відкрити розташування файлу
open-with-title = Чим ви бажаєте відкрити "{ $name }"?
browse-store = Пошукати в { $store }
other-apps = Інші застосунки
related-apps = Пов'язані застосунки
permanently-delete-question = Видалити назавжди
delete = Видалити
permanently-delete-warning = Ви впевнені, що бажаєте назавжди видалити { $target }? Цю дію відмінити неможливо.
set-executable-and-launch = Дозволити виконання та запустити
set-executable-and-launch-description = Ви впевнені, що бажаєте дозволити виконання файлу "{ $name }" та запустити його?
set-and-launch = Дозволити та запустити
open-with = Відкрити за допомогою
owner = Власник
group = Група
other = Інші
none = Нічого
execute-only = Тільки виконання
write-only = Тільки запис
write-execute = Запис та виконання
read-only = Тільки читання
read-execute = Читання та виконання
read-write = Читання та запис
read-write-execute = Читання, запис та виконання
favorite-path-error = Помилка при доступу до теки
favorite-path-error-description =
    Неможливо відкрити "{ $path }".
    Можливо за цим шляхом нічого немає, або у вас немає прав, щоб відкрити це.

    Бажаєте видалити це з бічної панелі?
keep = Залишити
add-network-drive = Додати мережевий диск
connect = Під'єднатися
connect-anonymously = Під'єднатися анонімно
connecting = З'єднання...
domain = Домен
enter-server-address = Введіть адресу серверу
network-drive-description =
    Адреси сервера включають в себе префікс з протоколом та саму адресу.
    Наприклад: ssh://192.168.0.1, ftp://[2001:db8::1]
network-drive-schemes =
    Доступні протоколи,префікс
    AppleTalk,afp://
    File Transfer Protocol,ftp:// або ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// або ssh://
    WebDAV,dav:// або davs://
network-drive-error = Неможливо отримати доступ до мережевого диску
password = Пароль
remember-password = Запам'ятати пароль
try-again = Спробувати знову
username = Ім'я користувача
progress = { $percent }%
progress-cancelled = { $percent }%, скасовано
progress-failed = { $percent }%, невдало
progress-paused = { $percent }%, призупинено
compressing =
    Стиснення { $items } { $items ->
        [one] об'єкту
       *[other] об'єктів
    } з "{ $from }" у "{ $to }" ({ $progress })...
compressed =
    Стиснено { $items } { $items ->
        [one] об'єкт
        [few] об'єкти
        [many] об'єктів
       *[other] об'єктів
    } з "{ $from }" у "{ $to }"
deleting =
    Видалення { $items } { $items ->
        [one] об'єкту
       *[other] об'єктів
    } з { trash } ({ $progress })...
deleted =
    Видалено { $items } { $items ->
        [one] об'єкт
        [few] об'єкти
        [many] об'єктів
       *[other] об'єктів
    } з { trash }
extracting =
    Видобування { $items } { $items ->
        [one] об'єкта
       *[other] об'єктів
    } з "{ $from }" у "{ $to }" ({ $progress })...
extracted =
    Видобуто { $items } { $items ->
        [one] об'єкт
        [few] об'єкти
        [many] об'єктів
       *[other] об'єктів
    } з "{ $from }" у "{ $to }"
setting-executable-and-launching = Надання дозволу на виконання "{ $name }" та запуск
set-executable-and-launched = Надано дозвіл на виконання та запущено "{ $name }"
selected-items = { $items } обраних елементів
setting-permissions = Встановлення дозволів для "{ $name }" на { $mode }
set-permissions = Встановлено дозволи { $mode } для "{ $name }"
show-details = Показати деталі
type = Тип: { $mime }
items = Об'єктів: { $items }
item-size = Об'єм: { $size }
item-created = Створено: { $created }
item-modified = Змінено: { $modified }
item-accessed = Доступ: { $accessed }
calculating = Обчислення...
single-click = Одне натискання миші для відкриття
type-to-search = Пошуковий рядок
type-to-search-recursive = Шукає поточну теку та усі підтеки
type-to-search-enter-path = Вводить шлях до теки або файлу
compress = Стиснути
delete-permanently = Видалити назавжди
eject = Витягнути
extract-here = Видобути
sort-by-trashed = Відсортувати за часом видалення
remove-from-recents = Видалити з нещодавніх
change-wallpaper = Змінити шпалери...
desktop-appearance = Вигляд стільниці...
display-settings = Налаштування дисплею...
reload-folder = Перезавантажити теку
gallery-preview = Попередній перегляд
sort = Сортування
sort-a-z = А-Я
sort-z-a = Я-А
sort-newest-first = Спочатку найновіші
sort-oldest-first = Спочатку найстаріші
sort-smallest-to-largest = Від найменшого до найбільшого
sort-largest-to-smallest = Від найбільшого до найменшого
permanently-deleting =
    Видалення { $items } { $items ->
        [one] об'єкта
       *[other] об'єктів
    } назавжди
permanently-deleted =
    Видалено { $items } { $items ->
        [one] об'єкт
        [few] об'єкта
        [many] об'єктів
       *[other] об'єктів
    } назавжди
removing-from-recents =
    Видалення { $items } { $items ->
        [one] об'єкта
       *[other] об'єктів
    } з { recents }
removed-from-recents =
    Видалено { $items } { $items ->
        [one] об'єкт
        [few] об'єкта
        [many] об'єктів
       *[other] об'єктів
    } з { recents }
