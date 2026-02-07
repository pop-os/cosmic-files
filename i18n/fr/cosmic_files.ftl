cosmic-files = Fichiers COSMIC
empty-folder = Dossier vide
empty-folder-hidden = Dossier vide (contient des éléments cachés)
no-results = Aucun résultat trouvé
filesystem = Système de fichiers
home = Dossier personnel
networks = Réseaux
notification-in-progress = Des opérations sur des fichiers sont en cours
trash = Corbeille
recents = Récents
undo = Annuler
today = Aujourd'hui
# Desktop view options
desktop-view-options = Options d'affichage du bureau...
show-on-desktop = Afficher sur le bureau
desktop-folder-content = Contenu du dossier du bureau
mounted-drives = Lecteurs montés
trash-folder-icon = Icône du dossier Corbeille
icon-size-and-spacing = Taille et espacement des icônes
icon-size = Taille des icônes
grid-spacing = Espacement de la grille
# List view
name = Nom
modified = Modifié
trashed-on = Mis à la corbeille
size = Taille
# Progress footer
details = Détails
dismiss = Ignorer le message
operations-running =
    { $running } { $running ->
        [one] opération
       *[other] opérations
    } en cours ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] opération
       *[other] opérations
    } en cours ({ $percent }%), { $finished } terminé...
pause = Pause
resume = Reprendre

# Dialogs


## Compress Dialog

create-archive = Créer une archive

## Extract Dialog

extract-password-required = Mot de passe requis
extract-to = Extraire vers...
extract-to-title = Extraire vers le dossier

## Empty Trash Dialog

empty-trash = Vider la corbeille
empty-trash-warning = Les éléments de la corbeille seront définitivement supprimés

## Mount Error Dialog

mount-error = Impossible d'accéder au lecteur

## New File/Folder Dialog

create-new-file = Créer un nouveau fichier
create-new-folder = Créer un nouveau dossier
file-name = Nom du fichier
folder-name = Nom du dossier
file-already-exists = Un fichier portant ce nom existe déjà
folder-already-exists = Un dossier portant ce nom existe déjà
name-hidden = Les noms commençant par "." seront cachés
name-invalid = Le nom ne peut pas être "{ $filename }"
name-no-slashes = Le nom ne peut pas contenir de slashs

## Open/Save Dialog

cancel = Annuler
create = Créer
open = Ouvrir
open-file = Ouvrir le fichier
open-folder = Ouvrir dossier
open-in-new-tab = Ouvrir dans un nouvel onglet
open-in-new-window = Ouvrir dans une nouvelle fenêtre
open-item-location = Ouvrir l'emplacement de l'élément
open-multiple-files = Ouvrir plusieurs fichiers
open-multiple-folders = Ouvrir plusieurs dossiers
save = Enregistrer
save-file = Enregistrer fichier

## Open With Dialog

open-with-title = Comment souhaitez-vous ouvrir "{ $name }" ?
browse-store = Parcourir { $store }

## Permanently delete Dialog

selected-items = Les { $items } éléments sélectionnés
permanently-delete-question = Supprimer définitivement ?
delete = Supprimer
permanently-delete-warning = { $target } sera définitivement supprimé. Cette action ne peut pas être annulée.

## Rename Dialog

rename-file = Renommer le fichier
rename-folder = Renommer le dossier

## Replace Dialog

replace = Remplacer
replace-title = "{ $filename }" existe déjà à cet endroit
replace-warning = Voulez-vous remplacer ce fichier par celui que vous enregistrez ? Cela écrasera son contenu.
replace-warning-operation = Voulez-vous remplacer ce fichier ? Cela écrasera son contenu.
original-file = Fichier d'origine
replace-with = Remplacer par
apply-to-all = Appliquer à tous
keep-both = Conserver les deux
skip = Ignorer

## Set as Executable and Launch Dialog

set-executable-and-launch = Définir comme exécutable et lancer
set-executable-and-launch-description = Voulez-vous définir "{ $name }" comme exécutable et le lancer ?
set-and-launch = Définir et lancer

## Metadata Dialog

open-with = Ouvrir avec
owner = Propriétaire
group = Groupe
other = Autre

### Mode 0

none = Aucun

### Mode 1 (unusual)

execute-only = Exécution seule

### Mode 2 (unusual)

write-only = Écriture seule

### Mode 3 (unusual)

write-execute = Écriture et exécution

### Mode 4

read-only = Lecture seule

### Mode 5

read-execute = Lecture et exécution

### Mode 6

read-write = Lecture et écriture

### Mode 7

read-write-execute = Lecture, écriture et exécution

## Favorite Path Error Dialog

favorite-path-error = Erreur lors de l'ouverture du répertoire
favorite-path-error-description =
    Impossible d'ouvrir "{ $path }"
    "{ $path }" n'existe peut-être pas ou vous n'avez pas la permission de l'ouvrir

    Voulez-vous le retirer de la barre latérale ?
remove = Supprimer
keep = Conserver

# Context Pages


## About


## Add Network Drive

add-network-drive = Ajouter un lecteur réseau
connect = Connecter
connect-anonymously = Se connecter anonymement
connecting = Connexion...
domain = Domaine
enter-server-address = Entrez l'adresse du serveur
network-drive-description =
    Les adresses de serveur incluent un préfixe de protocole et une adresse.
    Exemples : ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Protocoles disponibles,Préfixe
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Impossible d'accéder au lecteur réseau
password = Mot de passe
remember-password = Se souvenir du mot de passe
try-again = Essayer à nouveau
username = Nom d'utilisateur

## Operations

cancelled = Annulé
edit-history = Modifier l'historique
history = Historique
no-history = Aucun élément dans l'historique.
pending = En attente
progress = { $percent }%
progress-cancelled = { $percent }%, annulation
progress-paused = { $percent }%, en pause
failed = Échoué
complete = Terminé
compressing =
    Compression de { $items } { $items ->
        [one] élément
       *[other] éléments
    } depuis "{ $from }" vers "{ $to }" ({ $progress })...
compressed =
    { $items } { $items ->
        [one] élément compressé
       *[other] éléments compressés
    } depuis "{ $from }" vers "{ $to }"
copy_noun = Copier
creating = Création de "{ $name }" dans "{ $parent }"
created = "{ $name }" créé dans "{ $parent }"
copying =
    Copie de { $items } { $items ->
        [one] élément
       *[other] éléments
    } depuis "{ $from }" vers "{ $to }" ({ $progress })...
copied =
    { $items } { $items ->
        [one] élément copié
       *[other] éléments copiés
    } depuis "{ $from }" vers "{ $to }"
deleting =
    Suppression de { $items } { $items ->
        [one] élément
       *[other] éléments
    } depuis { trash } ({ $progress })...
deleted =
    { $items } { $items ->
        [one] élément supprimé
       *[other] éléments supprimés
    } depuis { trash }
emptying-trash = { trash } en cours de nettoyage ({ $progress })...
emptied-trash = { trash } vidée
extracting =
    Extraction de { $items } { $items ->
        [one] élément
       *[other] éléments
    } depuis "{ $from }" vers "{ $to }" ({ $progress })...
extracted =
    { $items } { $items ->
        [one] élément extrait
       *[other] éléments extraits
    } depuis "{ $from }" vers "{ $to }"
setting-executable-and-launching = Paramétrage de "{ $name }" comme exécutable et prêt à être lancé
set-executable-and-launched = Défini "{ $name }" comme exécutable et lancé
moving =
    Déplacement de { $items } { $items ->
        [one] élément
       *[other] éléments
    } depuis "{ $from }" vers "{ $to }" ({ $progress })...
moved =
    { $items } { $items ->
        [one] élément déplacé
       *[other] éléments déplacés
    } de "{ $from }" à "{ $to }"
permanently-deleting =
    Suppression définitive de { $items } { $items ->
        [one] élément
       *[other] éléments
    }
permanently-deleted =
    { $items } { $items ->
        [one] élément supprimé
       *[other] éléments supprimés
    } définitivement
renaming = Renommage de "{ $from }" en "{ $to }"
renamed = "{ $from }" renommé en "{ $to }"
restoring =
    Restauration de { $items } { $items ->
        [one] élément
       *[other] éléments
    } depuis la { trash } ({ $progress })...
restored =
    { $items } { $items ->
        [one] élément restauré
       *[other] éléments restaurés
    } depuis la { trash }
unknown-folder = Dossier inconnu

## Open with

menu-open-with = Ouvrir avec...
default-app = { $name } (défaut)

## Show details

show-details = Afficher les détails
type = Type : { $mime }
items = Éléments : { $items }
item-size = Taille : { $size }
item-created = Créé : { $created }
item-modified = Modifié : { $modified }
item-accessed = Consulté : { $accessed }
calculating = Calcul en cours...

## Settings

settings = Paramètres
single-click = Ouvrir en un clic

### Appearance

appearance = Apparence
theme = Thème
match-desktop = Assortir au bureau
dark = Sombre
light = Clair

### Type to Search

type-to-search = Tapez pour rechercher
type-to-search-recursive = Recherche dans le dossier actuel et tous les sous-dossiers
type-to-search-enter-path = Entrez le chemin du dossier ou du fichier
# Context menu
add-to-sidebar = Ajouter à la barre latérale
compress = Compresser
delete-permanently = Supprimer définitivement
extract-here = Extraire
new-file = Nouveau fichier...
new-folder = Nouveau dossier...
open-in-terminal = Ouvrir dans le terminal
move-to-trash = Déplacer vers la corbeille
restore-from-trash = Restaurer depuis la corbeille
remove-from-sidebar = Supprimer de la barre latérale
sort-by-name = Trier par nom
sort-by-modified = Trier par date de modification
sort-by-size = Trier par taille
sort-by-trashed = Trier par date de suppression

## Desktop

change-wallpaper = Changer le fond d'écran...
desktop-appearance = Apparence du bureau...
display-settings = Paramètres d'affichage...

# Menu


## File

file = Fichier
new-tab = Nouvel onglet
new-window = Nouvelle fenêtre
rename = Renommer...
close-tab = Fermer l'onglet
quit = Quitter

## Edit

edit = Modifier
cut = Couper
copy = Copier
paste = Coller
select-all = Sélectionner tout

## View

zoom-in = Zoomer
default-size = Taille par défaut
zoom-out = Dézoomer
view = Affichage
grid-view = Vue en grille
list-view = Vue en liste
show-hidden-files = Afficher les fichiers cachés
list-directories-first = Lister les répertoires en premier
gallery-preview = Aperçu de la galerie
menu-settings = Paramètres...
menu-about = À propos de Fichiers COSMIC...

## Sort

sort = Trier
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Le plus récent en premier
sort-oldest-first = Le plus ancien en premier
sort-smallest-to-largest = Du plus petit au plus grand
sort-largest-to-smallest = Du plus grand au plus petit
reload-folder = Actualiser le dossier
related-apps = Applications associées
removing-from-recents =
    Suppression de { $items } { $items ->
        [one] élément
       *[other] éléments
    } de { recents }
other-apps = Autres applications
set-permissions = Définir les permissions pour "{ $name }" à { $mode }
repository = Dépôt
support = Support
eject = Éjecter
remove-from-recents = Supprimer des récents
empty-trash-title = Vider la corbeille ?
setting-permissions = Définition des permissions pour "{ $name }" à { $mode }
removed-from-recents =
    { $items } { $items ->
        [one] élément supprimé
       *[other] éléments supprimés
    } de { recents }
progress-failed = { $percent }%, échec
type-to-search-select = Sélectionne le premier fichier ou dossier qui convient
pasted-image = Image collée
pasted-text = Text collé
pasted-video = Vidéo collée
