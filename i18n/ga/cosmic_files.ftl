cosmic-files = Comhaid COSMIC
empty-folder = Fillteán folamh
empty-folder-hidden = Fillteán folamh (tá míreanna folaithe ann)
no-results = Gan aon torthaí le fáil
filesystem = Córas comhad
home = Baile
networks = Líonraí
notification-in-progress = Tá oibríochtaí comhaid ar siúl.
trash = Bruscar
recents = Le Déanaí
undo = Cealaigh
today = Inniu

# Desktop view options
desktop-view-options = Roghanna radhairc deisce...
show-on-desktop = Taispeáin ar an Deasc
desktop-folder-content = Ábhar an fhillteáin deisce
mounted-drives = Tiomántáin mhonaithe
trash-folder-icon = Deilbhín an bhruscair
icon-size-and-spacing = Méid agus spásáil na ndeilbhíní
icon-size = Méid na ndeilbhíní
grid-spacing = Spásáil an ghreille

# List view
name = Ainm
modified = Athraithe
trashed-on = Curtha sa Bhruscar
size = Méid

# Progress footer
details = Sonraí
dismiss = Dún an teachtaireacht
operations-running = {$running} oibríocht ar siúl ({$percent}%)...
operations-running-finished = {$running} oibríocht ar siúl ({$percent}%), {$finished} críochnaithe...
pause = Sos
resume = Lean ar aghaidh

# Dialogs

## Compress Dialog
create-archive = Cruthaigh cartlann

## Extract Dialog
extract-password-required = Pasfhocal riachtanach

## Empty Trash Dialog
empty-trash = Folmhaigh an bruscar
empty-trash-warning = An bhfuil tú cinnte gur mian leat na míreanna go léir sa Bhruscar a scriosadh go buan?

## Mount Error Dialog
mount-error = Ní féidir rochtain a fháil ar an tiomántán

## New File/Folder Dialog
create-new-file = Cruthaigh comhad nua
create-new-folder = Cruthaigh fillteán nua
file-name = Ainm an chomhaid
folder-name = Ainm an fhillteáin
file-already-exists = Tá comhad leis an ainm sin ann cheana.
folder-already-exists = Tá fillteán leis an ainm sin ann cheana.
name-hidden = Beidh ainmneacha a thosaíonn le "." folaithe.
name-invalid = Ní féidir an t-ainm a bheith "{$filename}".
name-no-slashes = Ní féidir siombailí slasa a bheith san ainm.

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
open-with-title = Conas is mian leat "{$name}" a oscailt?
browse-store = Brabhsáil {$store}

## Rename Dialog
rename-file = Athainmnigh comhad
rename-folder = Athainmnigh fillteán

## Replace Dialog
replace = Ionadaigh
replace-title = Tá "{$filename}" ann sa suíomh seo cheana féin.
replace-warning = An bhfuil tú cinnte gur mian leat é a athsholáthar leis an gceann atá á shábháil agat? Scriosfar an t-ábhar atá ann cheana.
replace-warning-operation = An bhfuil tú cinnte gur mian leat é a athsholáthar? Scriosfar an t-ábhar atá ann cheana.
original-file = An comhad bunaidh
replace-with = Ionadaigh le
apply-to-all = Cuir i bhfeidhm ar gach ceann
keep-both = Coinnigh an dá cheann
skip = Scip

## Set as Executable and Launch Dialog
set-executable-and-launch = Socraigh mar inrite agus seol
set-executable-and-launch-description = An bhfuil tú ag iarraidh "{$name}" a dhéanamh inrite agus é a sheoladh?
set-and-launch = Socraigh agus seol

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
write-only = Scríofa amháin
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

# Context Pages

## About
git-description = Tiomantas Git {$hash} ar {$date}

## Add Network Drive
add-network-drive = Cuir tiomántán líonra leis
connect = Ceangail
connect-anonymously = Ceangail gan ainm
connecting = Ag ceangal...
domain = Fearann
enter-server-address = Enter server address
network-drive-description =
    Áirítear le seoltaí freastalaí réimír prótacail agus seoladh.
    Samplaí: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Prótacail atá ar fáil, Réimír
    AppleTalk,afp://
    File Transfer Protocol,ftp:// nó ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// nó ssh://
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
no-history = Níl aon mhír sa stair.
pending = Ar feitheamh
progress = {$percent}%
progress-cancelled = {$percent}%, cealaithe
progress-paused = {$percent}%, curtha ar shos
failed = Theip
complete = Críochnaithe
compressing = Á chomhbhrú {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó "{$from}" go "{$to}" ({$progress})...
compressed = Comhbhrúdh {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó "{$from}" go "{$to}"
copy_noun = Cóipeáil
creating = Á chruthú "{$name}" i "{$parent}"
created = Cruthaíodh "{$name}" i "{$parent}"
copying = Á chóipeáil {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó "{$from}" go "{$to}" ({$progress})...
copied = Cóipeáladh {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó "{$from}" go "{$to}"
emptying-trash = Á fholmhú {trash} ({$progress})...
emptied-trash = Folmhíodh {trash}
extracting = Á bhaint {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó "{$from}" go "{$to}" ({$progress})...
extracted = Bainíodh {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó "{$from}" go "{$to}"
setting-executable-and-launching = Á shocrú "{$name}" mar chomhad inrite agus á thosú
set-executable-and-launched = Socraíodh "{$name}" mar chomhad inrite agus tosaíodh é
moving = Á bhogadh {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó "{$from}" go "{$to}" ({$progress})...
moved = Bogadh {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó "{$from}" go "{$to}"
renaming = Á athainmniú "{$from}" go "{$to}"
renamed = Athainmníodh "{$from}" go "{$to}"
restoring = Á chur ar ais {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó {trash} ({$progress})...
restored = Cuireadh ar ais {$items} {$items ->
        [one] mhír
        *[other] míreanna
    } ó {trash}
unknown-folder = Fillteán anaithnid

## Open with
menu-open-with = Oscail le...
default-app = {$name} (réamhshocraithe)

## Show details
show-details = Taispeáin sonraí
type = Cineál: {$mime}
items = Míreanna: {$items}
item-size = Méid: {$size}
item-created = Cruthaithe: {$created}
item-modified = Athraithe: {$modified}
item-accessed = Rochtain: {$accessed}
calculating = Á ríomh...

## Settings
settings = Socruithe

### Appearance
appearance = Cuma
theme = Téama
match-desktop = Comhoiriúnaigh an deasc
dark = Dorcha
light = Geal

# Context menu
add-to-sidebar = Cuir leis an mbarra taoibh
compress = Comhbhrúigh
extract-here = Bain anseo
new-file = Comhad nua...
new-folder = Fillteán nua...
open-in-terminal = Oscail sa teirminéal
move-to-trash = Bog go dtí an bruscar
restore-from-trash = Athchóirigh ón mbruscar
remove-from-sidebar = Bain ón mbarra taoibh
sort-by-name = Sórtáil de réir ainm
sort-by-modified = Sórtáil de réir athraithe
sort-by-size = Sórtáil de réir méid
sort-by-trashed = Sórtáil de réir ama scriosta

## Desktop
change-wallpaper = Athraigh an páipéar balla...
desktop-appearance = Cuma na deisce...
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
zoom-in = Méadaigh
default-size = Méid réamhshocraithe
zoom-out = Laghdaigh
view = Amharc
grid-view = Amharc greille
list-view = Amharc liosta
show-hidden-files = Taispeáin comhaid fholaithe
list-directories-first = Liostaigh na heolairí ar dtús
gallery-preview = Réamhamharc gailearaí
menu-settings = Socruithe...
menu-about = Maidir le Comhaid COSMIC...

## Sort
sort = Sórtáil
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Is nuaí ar dtús
sort-oldest-first = Is sine ar dtús
sort-smallest-to-largest = Is lú go dtí an ceann is mó
sort-largest-to-smallest = Is mó go ceann is lú
