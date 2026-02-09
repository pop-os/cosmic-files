cosmic-files = Файли COSMIC
empty-folder = Порожня тека
empty-folder-hidden = Порожня тека (містить приховані елементи)
filesystem = Файлова система
home = Домівка
trash = Смітник
recents = Нещодавні
undo = Відмінити
# List view
name = Назва
modified = Змінено
size = Розмір

# Dialogs


## Empty Trash Dialog

empty-trash = Спорожнити смітник
empty-trash-warning = Елементи зі смітника будуть остаточно видалені

## New File/Folder Dialog

create-new-file = Створити новий файл
create-new-folder = Створити нову теку
file-name = Назва файлу
folder-name = Назва теки
file-already-exists = Файл з такою назвою вже існує
folder-already-exists = Тека з такою назвою вже існує
name-hidden = Назви, що починаються з «.», приховано
name-invalid = Назва не може бути «{ $filename }»
name-no-slashes = Назва не може містити скісні риски

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
replace-title = «{ $filename }» вже існує в цій теці
replace-warning = Бажаєте замінити файл тим, що зберігаєте? Після заміни його вміст буде перезаписано.
replace-warning-operation = Бажаєте замінити його? Це перезапише його вміст.
original-file = Початковий файл
replace-with = Замінити на
apply-to-all = Застосувати до всіх
keep-both = Залишити обидва
skip = Пропустити

# Context Pages


## About


## Operations

edit-history = Історія дій
history = Історія
no-history = Історія порожня.
pending = Очікується
failed = Не виконано
complete = Завершено
copy_noun = Копіювати
creating = Створення «{ $name }» в «{ $parent }»
created = Створено «{ $name }» у «{ $parent }»
copying =
    Копіювання { $items } { $items ->
        [one] елемента
       *[other] елементів
    } з «{ $from }» в «{ $to }» ({ $progress })...
copied =
    Скопійовано { $items } { $items ->
        [one] елемент
       *[other] елеменів
    } з «{ $from }» в «{ $to }»
emptying-trash = Спорожнення { trash } ({ $progress })...
emptied-trash = Спорожнено { trash }
moving =
    Переміщення { $items } { $items ->
        [one] елемента
       *[other] елементів
    } з «{ $from }» в «{ $to }» ({ $progress })...
moved =
    Переміщено { $items } { $items ->
        [one] елемент
       *[other] елементи
    } з «{ $from }» в «{ $to }»
renaming = Перейменування «{ $from }» на «{ $to }»
renamed = Перейменовано «{ $from }» на «{ $to }»
restoring =
    Відновлення { $items } { $items ->
        [one] елемента
       *[other] елементів
    } з { trash } ({ $progress })...
restored =
    Відновлено { $items } { $items ->
        [one] елемент
       *[other] елементи
    } з { trash }
unknown-folder = невідома тека

## Open with

menu-open-with = Відкрити за допомогою...
default-app = { $name } (звичайний)

## Properties


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
move-to-trash = Пересунути до смітника
restore-from-trash = Відновити зі смітника
remove-from-sidebar = Вилучити з бічної панелі
sort-by-name = Упорядкувати за назвою
sort-by-modified = Упорядкувати за зміною
sort-by-size = Упорядкувати за розміром

# Menu


## File

file = Файл
new-tab = Нова вкладка
new-window = Нове вікно
rename = Перейменувати...
close-tab = Закрити вкладку
quit = Вийти

## Edit

edit = Редагувати
cut = Вирізати
copy = Копіювати
paste = Вставити
select-all = Вибрати все

## View

zoom-in = Збільшити
default-size = Стандартний розмір
zoom-out = Зменшити
view = Вид
grid-view = Перегляд таблицею
list-view = Перегляд списком
show-hidden-files = Показати приховані файли
list-directories-first = Теки спочатку
menu-settings = Налаштування...
menu-about = Про Файли COSMIC...
repository = Сховище
support = Підтримка
details = Деталі
dismiss = Закрити повідомлення
remove = Вилучити
cancelled = Скасовані
no-results = Нічого не знайдено
networks = Мережі
notification-in-progress = Триває обробка файлів
today = Сьогодні
desktop-view-options = Параметри вигляду стільниці...
show-on-desktop = Показувати на стільниці
desktop-folder-content = Вміст теки Стільниця
mounted-drives = Змонтовані диски
trash-folder-icon = Піктограма Смітника
icon-size-and-spacing = Розмір піктограм і відстань між ними
icon-size = Розмір піктограм
grid-spacing = Відстань між піктограмами
trashed-on = У смітнику
operations-running =
    { $running } { $running ->
        [one] операція
       *[other] операції
    } виконується ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] операція
       *[other] операціі
    } виконується ({ $percent }%), { $finished } завершено...
pause = Призупинити
resume = Продовжити
create-archive = Створити архів
extract-password-required = Потрібен пароль
extract-to = Видобути до...
extract-to-title = Видобути до теки
mount-error = Доступ до диска відсутній
create = Створити
open-item-location = Відкрити розташування елемента
open-with-title = Як ви бажаєте відкрити «{ $name }»?
browse-store = Переглянути { $store }
other-apps = Інші застосунки
related-apps = Пов'язані застосунки
permanently-delete-question = Остаточно видалити?
delete = Видалити
permanently-delete-warning = { $target } буде остаточно видалено. Цю дію не можна скасувати.
set-executable-and-launch = Зробити виконуваним і запустити
set-executable-and-launch-description = Бажаєте зробити "{ $name }" виконуваним і запустити його?
set-and-launch = Зробити і запустити
open-with = Відкрити за допомогою
owner = Власник
group = Група
other = Інші
none = Немає прав
execute-only = Тільки виконання
write-only = Тільки запис
write-execute = Запис і виконання
read-only = Тільки перегляд
read-execute = Перегляд і виконання
read-write = Перегляд і запис
read-write-execute = Перегляд, запис і виконання
favorite-path-error = Помилка при відкритті каталогу
favorite-path-error-description =
    Не вдалося відкрити «{ $path }»
    «{ $path }» можливо не існує або у вас немає прав на відкриття

    Вилучити з бічної панелі?
keep = Залишити
add-network-drive = Додати мережевий диск
connect = З'єднати
connect-anonymously = З'єднатись анонімно
connecting = З'єднання…
domain = Домен
enter-server-address = Введіть адресу сервера
network-drive-description =
    Серверні адреси містять префікс протоколу і саму адресу.
    Наприклад: ssh://192.168.0.1, ftp://[2001:db8::1]
network-drive-schemes =
    Доступні протоколи,Префікс
    AppleTalk,afp://
    Протокол Передавання Файлів,ftp:// або ftps://
    Мережева Файлова Система,nfs://
    Серверний Блок Повідомлень,smb://
    Протокол Передавання Файлів SSH,sftp:// або ssh://
    WebDAV,dav:// або davs://
network-drive-error = Доступ до мережевого диска відсутній
password = Пароль
remember-password = Запам'ятати пароль
try-again = Спробувати знову
username = Ім'я користувача
progress = { $percent }%
progress-cancelled = { $percent }%, скасовано
progress-failed = { $percent }%, не вдалося
progress-paused = { $percent }%, призупинено
compressing =
    Стиснення { $items } { $items ->
        [one] елемента
       *[other] елементів
    } з "{ $from }" до "{ $to }" ({ $progress })...
compressed =
    Стиснуто { $items } { $items ->
        [one] елемент
       *[other] елементи
    } з "{ $from }" до "{ $to }"
deleting =
    Видалення { $items } { $items ->
        [one] елемента
       *[other] елементів
    } з { trash } ({ $progress })...
deleted =
    Видалено { $items } { $items ->
        [one] елемент
       *[other] елементи
    } з { trash }
extracting =
    Видобування { $items } { $items ->
        [one] елемента
       *[other] елементів
    } з «{ $from }» в «{ $to }» ({ $progress })...
extracted =
    Видобуто { $items } { $items ->
        [one] елемент
       *[other] елементи
    } з «{ $from }» в «{ $to }»
setting-executable-and-launching = Надання «{ $name }» прав на виконання та запуск
set-executable-and-launched = «{ $name }» надано права на виконання і відкрито
selected-items = Вибрані { $items } елементи
setting-permissions = Надання прав { $mode } для «{ $name }»
set-permissions = Надано права { $mode } для «{ $name }»
show-details = Показати подробиці
type = Тип: { $mime }
items = Елементів: { $items }
item-size = Розмір: { $size }
item-created = Створено: { $created }
item-modified = Змінено: { $modified }
item-accessed = Дата доступу: { $accessed }
calculating = Обчислення...
single-click = Відкривати одним клацанням
type-to-search = Введіть для пошуку
type-to-search-recursive = Шукає у поточній теці та всіх підтеках
type-to-search-enter-path = Вводить шлях до каталогу або файлу
compress = Стиснути
delete-permanently = Остаточно видалити
eject = Безпечно вилучити
extract-here = Видобути
sort-by-trashed = Упорядкувати за часом видалення
remove-from-recents = Вилучити з нещодавніх
change-wallpaper = Змінити зображення тла...
desktop-appearance = Вигляд стільниці...
display-settings = Налаштування дисплея...
reload-folder = Оновити теку
gallery-preview = Попередній перегляд галереї
sort = Упорядкувати
sort-a-z = А-Я
sort-z-a = Я-А
sort-newest-first = Спочатку найновіші
sort-oldest-first = Спочатку найстаріші
sort-smallest-to-largest = Від найменшого до найбільшого
sort-largest-to-smallest = Від найбільшого до найменшого
permanently-deleting =
    Остаточне вилучення { $items } { $items ->
        [one] елемента
       *[other] елементів
    }
permanently-deleted =
    Остаточно вилучено { $items } { $items ->
        [one] елемент
       *[other] елементи
    }
removing-from-recents =
    Вилучення { $items } { $items ->
        [one] елемента
       *[other] елементів
    } з { recents }
removed-from-recents =
    Вилучено { $items } { $items ->
        [one] елемент
       *[other] елементи
    } з { recents }
empty-trash-title = Спорожити смітник?
type-to-search-select = Вибирає перший відповідний файл або папку
pasted-image = Вставлене Зображення
pasted-text = Вставлений Текст
pasted-video = Вставлене Видиво
