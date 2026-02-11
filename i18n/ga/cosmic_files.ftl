cosmic-files = Comhaid COSMIC
empty-folder = Fillteán folamh
empty-folder-hidden = Fillteán folamh (tá míreanna folaithe ann)
no-results = Níor aimsíodh aon torthaí
filesystem = Córas comhad
home = Baile
networks = Líonraí
notification-in-progress = Tá oibríochtaí comhaid ar siúl
trash = Bruscar
recents = Le Déanaí
undo = Cuir ar ceal
today = Inniu
# Desktop view options
desktop-view-options = Roghanna radhairc deisce...
show-on-desktop = Taispeáin ar an deasc
desktop-folder-content = Ábhar fillteáin deisce
mounted-drives = Tiomántáin mhonaithe
trash-folder-icon = Deilbhín fillteáin bruscair
icon-size-and-spacing = Méid agus spásáil na ndeilbhíní
icon-size = Méid na ndeilbhíní
grid-spacing = Spásáil an ghreille
# List view
name = Ainm
modified = Mionathraithe
trashed-on = Curtha sa bhruscar
size = Méid
# Progress footer
details = Sonraí
dismiss = Diúltaigh an teachtaireacht
operations-running =
    { $running } { $running ->
        [one] oibríocht
       *[other] oibríochtaí
    } ag rith ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] oibríocht
       *[other] oibríochtaí
    } ag rith ({ $percent }%), { $finished } críochnaithe...
pause = Sos
resume = Tosaigh arís

# Dialogs


## Compress Dialog

create-archive = Cruthaigh cartlann

## Extract Dialog

extract-password-required = Pasfhocal riachtanach
extract-to = Asbhain go...
extract-to-title = Asbhain go fillteán

## Empty Trash Dialog

empty-trash = Folmhaigh an bruscar
empty-trash-warning = Scriosfar míreanna sa bhfillteán Bruscair go buan

## Mount Error Dialog

mount-error = Ní féidir rochtain a fháil ar an tiomántán

## New File/Folder Dialog

create-new-file = Cruthaigh comhad nua
create-new-folder = Cruthaigh fillteán nua
file-name = Ainm comhaid
folder-name = Ainm fillteáin
file-already-exists = Tá comhad leis an ainm sin ann cheana féin
folder-already-exists = Tá fillteán leis an ainm sin ann cheana féin
name-hidden = Beidh ainmneacha ag tosú le "." i bhfolach
name-invalid = Ní féidir an t-ainm a bheith "{ $filename }"
name-no-slashes = Ní féidir slaiseanna a bheith san ainm

## Open/Save Dialog

cancel = Cealaigh
create = Cruthaigh
open = Oscail
open-file = Oscail comhad
open-folder = Oscail fillteán
open-in-new-tab = Oscail i gcluaisín nua
open-in-new-window = Oscail i bhfuinneog nua
open-item-location = Oscail suíomh na míre
open-multiple-files = Oscail ilchomhaid
open-multiple-folders = Oscail ilfhillteáin
save = Sábháil
save-file = Sábháil comhad

## Open With Dialog

open-with-title = Conas is mian leat "{ $name }" a oscailt?
browse-store = Brabhsáil { $store }

## Rename Dialog

rename-file = Athainmnigh comhad
rename-folder = Athainmnigh fillteán

## Replace Dialog

replace = Cuir in ionad
replace-title = Tá "{ $filename }" ann sa suíomh seo cheana féin
replace-warning = An bhfuil tú cinnte gur mian leat é a chur in ionad leis an gceann atá á shábháil agat? Scríobhfar an t-ábhar nua thairis ar an ábhar atá ann cheana.
replace-warning-operation = An bhfuil tú cinnte gur mian leat é a chur in ionad? Scríobhfar an t-ábhar nua thairis ar an ábhar atá ann cheana.
original-file = Comhad bunaidh
replace-with = Cuir in ionad le
apply-to-all = Cuir i bhfeidhm ar gach ceann
keep-both = Coinnigh an dá cheann
skip = Scipeáil

## Set as Executable and Launch Dialog

set-executable-and-launch = Socraigh mar inrite agus lainseáil
set-executable-and-launch-description = Ar mhaith leat "{ $name }" a shocrú mar chomhad inrite agus é a lainseáil?
set-and-launch = Socraigh agus lainseáil

## Metadata Dialog

open-with = Oscail le
owner = Úinéir
group = Grúpa
other = Eile

### Mode 0

none = Dada

### Mode 1 (unusual)

execute-only = Inrite amháin

### Mode 2 (unusual)

write-only = Scríobh amháin

### Mode 3 (unusual)

write-execute = Scríobh agus inrite

### Mode 4

read-only = Léamh amháin

### Mode 5

read-execute = Léamh agus inrite

### Mode 6

read-write = Léamh agus scríobh

### Mode 7

read-write-execute = Léamh, scríobh, agus inrite

## Favorite Path Error Dialog

favorite-path-error = Earráid ag oscailt an eolaire
favorite-path-error-description =
    Ní féidir "{ $path }" a oscailt
    B'fhéidir nach bhfuil "{ $path }" ann nó b'fhéidir nach bhfuil cead agat é a oscailt

    Ar mhaith leat é a bhaint den bharra taoibh?
remove = Bain
keep = Coimeád

# Context Pages


## About


## Add Network Drive

add-network-drive = Cuir tiomántán líonra leis
connect = Ceangail
connect-anonymously = Ceangail gan ainm
connecting = Ag ceangal...
domain = Fearann
enter-server-address = Cuir isteach seoladh an fhreastalaí
network-drive-description =
    Áirítear le seoltaí freastalaí réimír prótacail agus seoladh.
    Samplaí: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Prótacail atá ar fáil, Réimír
    AppleTalk,afp://
    Prótacal Aistrithe Comhad,ftp:// nó ftps://
    Córas Comhad Líonra,nfs://
    Bloc Teachtaireachtaí Freastalaí,smb://
    Prótacal Aistrithe Comhad SSH,sftp:// nó ssh://
    WebDav,dav:// nó davs://
network-drive-error = Ní féidir teacht ar an tiomántán líonra
password = Pasfhocal
remember-password = Cuimhnigh pasfhocal
try-again = Bain triail eile as
username = Ainm úsáideora

## Operations

cancelled = Cealaithe
edit-history = Cuir stair in eagar
history = Stair
no-history = Gan aon mhíreanna sa stair.
pending = Ar feitheamh
progress = { $percent }%
progress-cancelled = { $percent }%, cealaithe
progress-paused = Curtha ar shos ag { $percent }%
failed = Theip
complete = Críochnaithe
compressing =
    Á chomhbhrú { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó "{ $from }" go "{ $to }" ({ $progress })...
compressed =
    Comhbhrúdh { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó "{ $from }" go "{ $to }"
copy_noun = Cóipeáil
creating = Á chruthú "{ $name }" i "{ $parent }"
created = Cruthaíodh "{ $name }" i "{ $parent }"
copying =
    Ag cóipeáil { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó "{ $from }" go "{ $to }" ({ $progress })...
copied =
    Cóipeáilte { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó "{ $from }" go "{ $to }"
deleting =
    Ag scriosadh { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó { trash } ({ $progress })...
deleted =
    Scriosta { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó { trash }
emptying-trash = Ag folmhú { trash } ({ $progress })...
emptied-trash = Folmhaíodh an { trash }
extracting =
    Ag asbhaint { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó "{ $from }" go "{ $to }" ({ $progress })...
extracted =
    Asbhainte { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó "{ $from }" go "{ $to }"
setting-executable-and-launching = Ag socrú "{ $name }" mar inrite agus ag lainseáil
set-executable-and-launched = Socraigh "{ $name }" mar inrite agus lainseáilte
moving =
    Ag bogadh { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó "{ $from }" go "{ $to }" ({ $progress })...
moved =
    Bogadh { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó "{ $from }" go "{ $to }"
renaming = Ag athainmniú "{ $from }" go "{ $to }"
renamed = Athainmnithe "{ $from }" go "{ $to }"
restoring =
    Ag athchóiriú{ $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó { trash } ({ $progress })...
restored =
    Athchóirithe { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó { trash }
unknown-folder = Fillteán anaithnid

## Open with

menu-open-with = Oscail le...
default-app = { $name } (réamhshocraithe)

## Show details

show-details = Taispeáin sonraí
type = Cineál: { $mime }
items = Míreanna: { $items }
item-size = Méid: { $size }
item-created = Cruthaithe: { $created }
item-modified = Mionathraithe: { $modified }
item-accessed = Rochtainte: { $accessed }
calculating = Á ríomh...

## Settings

settings = Socruithe
single-click = Cliceáil amháin le hoscailt

### Appearance

appearance = Cuma
theme = Téama
match-desktop = Meaitseáil deasc
dark = Dorcha
light = Solas

### Type to Search

type-to-search = Clóscríobh le cuardach
type-to-search-recursive = Cuardaíonn sé an fillteán reatha agus na fo-fhillteáin go léir
type-to-search-enter-path = Iontrálann sé seo an cosán chuig an eolaire nó an comhad
# Context menu
add-to-sidebar = Cuir leis an mbarra taoibh
compress = Comhbhrúigh
delete-permanently = Scrios go buan
extract-here = Asbhain
new-file = Comhad nua...
new-folder = Fillteán nua...
open-in-terminal = Oscail sa teirminéal
move-to-trash = Bog go dtí an bruscar
restore-from-trash = Athchóirigh ón mbruscar
remove-from-sidebar = Bain ón mbarra taoibh
sort-by-name = Sórtáil de réir ainm
sort-by-modified = Sórtáil de réir modhnaithe
sort-by-size = Sórtáil de réir méid
sort-by-trashed = Sórtáil de réir am scriosta

## Desktop

change-wallpaper = Athraigh cúlbhrat...
desktop-appearance = Cuma deisce...
display-settings = Socruithe taispeána...

# Menu


## File

file = Comhad
new-tab = Cluaisín nua
new-window = Fuinneog nua
rename = Athainmnigh...
close-tab = Dún cluaisín
quit = Scoir

## Edit

edit = Eagar
cut = Gearr
copy = Cóipeáil
paste = Greamaigh
select-all = Roghnaigh gach ceann

## View

zoom-in = Súmáil isteach
default-size = Méid réamhshocraithe
zoom-out = Súmáil amach
view = Amharc
grid-view = Amharc greille
list-view = Amharc liosta
show-hidden-files = Taispeáin comhaid fholaithe
list-directories-first = Liostaigh eolairí ar dtús
gallery-preview = Réamhamharc gailearaí
menu-settings = Socruithe...
menu-about = Maidir le Comhaid COSMIC...

## Sort

sort = Sórtáil
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Is nuaí ar dtús
sort-oldest-first = Is sine ar dtús
sort-smallest-to-largest = Ón gceann is lú go dtí an ceann is mó
sort-largest-to-smallest = Ón gceann is mó go dtí an ceann is lú
repository = Stór
support = Tacaíocht
other-apps = Feidhmchláir eile
related-apps = Feidhmchláir ghaolmhara
selected-items = Na { $items } míreanna roghnaithe
permanently-delete-question = Scriosadh go buan?
delete = Scrios
permanently-delete-warning = Scriosfar { $target } go buan. Ní féidir an gníomh seo a chealú.
progress-failed = Theip ar { $percent }%
setting-permissions = Ceadanna á socrú do "{ $name }" go { $mode }
set-permissions = Socraigh ceadanna do "{ $name }" go { $mode }
permanently-deleting =
    Ag scriosadh go buan { $items } { $items ->
        [one] mhír
       *[other] míreanna
    }
permanently-deleted =
    Scriosta go buan { $items } { $items ->
        [one] mhír amháin
       *[other] míreanna
    }
removing-from-recents =
    Ag baint { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó { recents }
removed-from-recents =
    Bainte { $items } { $items ->
        [one] mhír
       *[other] míreanna
    } ó { recents }
eject = Díchuir
remove-from-recents = Bain as na cinn is déanaí
reload-folder = Athlódáil an fillteán
empty-trash-title = Folmhaigh an bruscar?
type-to-search-select = Roghnaíonn an chéad chomhad nó fillteán comhoiriúnach
pasted-image = Íomhá ghreamaithe
pasted-text = Téacs greamaithe
pasted-video = Físeán greamaithe
