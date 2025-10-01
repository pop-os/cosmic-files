cosmic-files = COSMIC Dosyalar
empty-folder = Boş klasör
empty-folder-hidden = Boş klasör (gizli ögeler içerir)
no-results = Sonuç bulunamadı
filesystem = Dosya sistemi
home = Ev
networks = Ağlar
notification-in-progress = Dosya işlemi devam etmekte.
trash = Çöp
recents = Son kullanılanlar
undo = Geri al
today = Bugün
# Desktop view options
desktop-view-options = Masaüstü görünüm seçenekleri...
show-on-desktop = Masaüstünde göster
desktop-folder-content = Masaüstü klasörü içeriği
mounted-drives = Bağlı sürücüler
trash-folder-icon = Çöp klasörü simgesi
icon-size-and-spacing = Simge boyutu ve aralığı
icon-size = Simge boyutu
# List view
name = Ad
modified = Değiştirilme
trashed-on = Çöpe atılma
size = Boyut
# Progress footer
details = Detaylar
dismiss = Mesajı kapat
operations-running =
    { $running } { $running ->
        [one] işlem
       *[other] işlemler
    } çalışıyor ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] işlem
       *[other] işlemler
    } çalışıyor ({ $percent }%), { $finished } bitti...
pause = Duraklat
resume = Devam et

# Dialogs


## Compress Dialog

create-archive = Arşivle

## Empty Trash Dialog

empty-trash = Çöpü boşalt
empty-trash-warning = Çöpteki bütün ögeleri kalıcı olarak silmek istediğine emin misiniz?

## Mount Error Dialog

mount-error = Sürücüye erişilemedi

## New File/Folder Dialog

create-new-file = Yeni dosya oluştur
create-new-folder = Yeni klasör oluştur
file-name = Dosya adı
folder-name = Klasör adı
file-already-exists = Bu adda bir dosya zaten var.
folder-already-exists = Bu adda bir klasör zaten var.
name-hidden = "." ile başlayan adlar gizlenecek.
name-invalid = Ad "{ $filename }" olamaz.
name-no-slashes = Ad eğik çizgi içeremez.

## Open/Save Dialog

cancel = Vazgeç
create = Oluştur
open = Aç
open-file = Dosya aç
open-folder = Klasör aç
open-in-new-tab = Yeni sekmede aç
open-in-new-window = Yeni pencerede aç
open-item-location = Öge konumunu aç
open-multiple-files = Birden fazla dosyayı aç
open-multiple-folders = Birden fazla klasörü aç
save = Kaydet
save-file = Dosyayı kaydet

## Open With Dialog

open-with-title = "{ $name }" dosyasını nasıl açmak istersiniz?
browse-store = { $store }'sını gezin

## Rename Dialog

rename-file = Dosyayı yeniden adlandır
rename-folder = Klasörü yeniden adlandır

## Replace Dialog

replace = Değiştir
replace-title = "{ $filename }" bu konumda zaten var.
replace-warning = Kaydettiğiniz dosya ile değiştirmek istiyor musunuz? Değiştirmek içeriğinin üzerine yazacak.
replace-warning-operation = Değiştirmek istiyor musunuz? Değiştirmek içeriğinin üzerine yazacak.
original-file = Orijinal dosya
replace-with = Değiştir
apply-to-all = Tümüne uygula
keep-both = İkisini de sakla
skip = Atla

## Set as Executable and Launch Dialog

set-executable-and-launch = Çalıştırılabilir olarak ayarla ve başlat
set-executable-and-launch-description = "{ $name }" dosyasını çalıştırılabilir olarak ayarlayıp başlatmak istiyor musunuz?
set-and-launch = Ayarla ve başlat

## Metadata Dialog

owner = Sahip
group = Grup
other = Diğer
read = Okuma
write = Yazma
execute = Çalıştırma

# Context Pages


## About

git-description = { $date } tarihli git commiti { $hash }

## Add Network Drive

add-network-drive = Ağ sürücüsü ekle
connect = Bağlan
connect-anonymously = Anonim olarak bağlan
connecting = Bağlanılıyor...
domain = Alan
enter-server-address = Sunucu adresi girin
network-drive-description =
    Sunucu adresleri protokol ön eki ve adres içerir.
    Örneğin: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Kullanılabilir protokoller,ön eki
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Ağ aygıtına erişilemedi
password = Şifre
remember-password = Parolayı hatırla
try-again = Tekrar dene
username = Kullanıcı adı

## Operations

cancelled = İptal edildi
edit-history = Geçmişi düzenle
history = Geçmiş
no-history = Geçmişte öge bulunmuyor.
pending = Askıda
progress = %{ $percent }
progress-cancelled = %{ $percent }, iptal edildi
progress-paused = %{ $percent }, duraklatıldı
failed = Başarısız
complete = Tamamlandı
compressing =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } "{ $from }" den "{ $to }" nesnesine sıkıştırılıyor ({ $progress })...
compressed =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den "{ $from }" e "{ $to }"
copy_noun = Kopyala
creating = "{ $parent }" dizininde "{ $name }" oluşturuluyor
created = "{ $parent }" dizininde "{ $name }" oluşturuldu
copying =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den "{ $from }" e "{ $to }" kopyalanıyor ({ $progress })...
copied =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den "{ $from }" e "{ $to }" kopyalandı
emptying-trash = { trash } boşaltılıyor ({ $progress })...
emptied-trash = { trash } boşaltıldı
extracting =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den "{ $from }" e "{ $to }" çıkartılıyor ({ $progress })...
extracted =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den "{ $from }" e "{ $to }" çıkartıldı
setting-executable-and-launching = "{ $name }" çalıştırılabilir olarak ayarlanıp başlatılıyor
set-executable-and-launched = "{ $name }" çalıştırılabilir olarak ayarlanıp başlatıldı
moving =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den "{ $from }" e "{ $to }" taşınıyor ({ $progress })...
moved =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den "{ $from }" e "{ $to }" taşındı
renaming = "{ $from }" adı "{ $to }" olarak değiştiriliyor
renamed = "{ $from }" adı "{ $to }" olarak değiştirildi
restoring =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den { trash } geri yükleniyor ({ $progress })...
restored =
    { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } den { trash } geri yüklendi
unknown-folder = bilinmeyen klasör

## Open with

menu-open-with = Birlikte aç...
default-app = { $name } (varsayılan)

## Show details

show-details = Detayları göster
type = Tür: { $mime }
items = Öge sayısı: { $items }
item-size = Boyut: { $size }
item-created = Oluşturuldu: { $created }
item-modified = Düzenlendi: { $modified }
item-accessed = Erişildi: { $accessed }
calculating = Hesaplanıyor...

## Settings

settings = Ayarlar

### Appearance

appearance = Görünüm
theme = Tema
match-desktop = Masaüstü stilini takip et
dark = Karanlık
light = Aydınlık
# Context menu
add-to-sidebar = Kenar çubuğuna ekle
compress = Sıkıştır
extract-here = Çıkar
new-file = Yeni dosya...
new-folder = Yeni klasör...
open-in-terminal = Uçbirimde aç
move-to-trash = Çöpe taşı
restore-from-trash = Çöpten geri yükle
remove-from-sidebar = Kenar çubuğundan kaldır
sort-by-name = Ada göre sırala
sort-by-modified = Düzenlenme tarihine göre sırala
sort-by-size = Boyuta göre sırala
sort-by-trashed = Silme tarihine göre sırala

## Desktop

change-wallpaper = Arka planı değiştir...
desktop-appearance = Masaüstü görünümü...
display-settings = Görüntü ayarları...

# Menu


## File

file = Dosya
new-tab = Yeni sekme
new-window = Yeni pencere
rename = Yeniden adlandır...
close-tab = Sekmeyi kapat
quit = Çıkış

## Edit

edit = Düzenle
cut = Kes
copy = Kopyala
paste = Yapıştır
select-all = Tümünü seç

## View

zoom-in = Yakınlaştır
default-size = Varsayılan boyut
zoom-out = Uzaklaştır
view = Görünüm
grid-view = Tablo görünümü
list-view = Liste görünümü
show-hidden-files = Gizli dosyaları göster
list-directories-first = Önce dizinleri listele
gallery-preview = Galeri ön izlemesi
menu-settings = Ayarlar...
menu-about = COSMIC Dosyalar hakkında...

## Sort

sort = Sırala
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Önce en yeni
sort-oldest-first = Önce en eski
sort-smallest-to-largest = En küçükten en büyüğe
sort-largest-to-smallest = En büyükten en küçüğe
repository = Depo
support = Destek
remove = Kaldır
grid-spacing = Izgara aralığı
extract-password-required = Şifre gerekli
extract-to = Çıkart...
extract-to-title = Klasöre çıkar
other-apps = Diğer uygulamalar
related-apps = İlgili uygulamalar
selected-items = { $items } seçili öğeler
permanently-delete-question = Kalıcı olarak sil
delete = Sil
permanently-delete-warning = { $target } kalıcı olarak silmek istediğinizden emin misiniz? Bu işlem geri alınamaz.
open-with = Birlikte aç
none = Yok
execute-only = Yalnızca çalıştırma
write-only = Yalnızca yazma
write-execute = Yazma ve çalıştırma
read-only = Yanlızca okuma
read-execute = Okuma ve çalıştırma
read-write = Okuma ve yazma
read-write-execute = Okuma, yazma ve çalıştırma
favorite-path-error = Dizin açılırken hata oluştu
favorite-path-error-description =
    "{ $path }" açılamıyor.
    Mevcut olmayabilir veya açma izniniz olmayabilir.

    Kenar çubuğundan kaldırmak ister misiniz?
keep = Tut
progress-failed = { $percent }%, başarısız oldu
deleting =
    Çöp kutusundan { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } { trash } ({ $progress }) siliniyor...
deleted =
    Çöp kutusundan { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } { trash } silindi
setting-permissions = "{ $name }" için izinler { $mode } olarak ayarlanıyor
set-permissions = "{ $name }" için izinleri { $mode } olarak ayarla
permanently-deleting =
    Kalıcı olarak { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } siliniyor
permanently-deleted =
    Kalıcı olarak { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } silindi
removing-from-recents =
    Şuradan { $items } { $items ->
        [one] nesne
       *[other] nesneler
    } { recents } kaldırılıyor
removed-from-recents =
    Şuradan { $items } { $items ->
        [one] nesne
       *[other] nesnenler
    } { recents } kaldırıldı
single-click = Açmak için tek tıklama
type-to-search = Aramak için yaz
type-to-search-recursive = Geçerli klasörü ve tüm alt klasörleri arar
type-to-search-enter-path = Dizin veya dosyanın yolunu girer
delete-permanently = Kalıcı olarak sil
eject = Çıkart
remove-from-recents = Son kullanılanlardan kaldır
reload-folder = Klasörü yeniden yükle
