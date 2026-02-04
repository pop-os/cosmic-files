cosmic-files = COSMIC Files
empty-folder = Empty folder
empty-folder-hidden = Empty folder (has hidden items)
no-results = No results found
filesystem = Filesystem
home = Home
networks = Networks
notification-in-progress = File operations are in progress
trash = Trash
recents = Recents
undo = Undo
today = Today

# Desktop view options
desktop-view-options = Desktop view options...
show-on-desktop = Show on Desktop
desktop-folder-content = Desktop folder content
mounted-drives = Mounted drives
trash-folder-icon = Trash folder icon
icon-size-and-spacing = Icon size and spacing
icon-size = Icon size
grid-spacing = Grid spacing

# List view
name = Name
modified = Modified
trashed-on = Trashed
size = Size

# Progress footer
details = Details
dismiss = Dismiss message
operations-running = {$running} {$running ->
    [one] operation
    *[other] operations
  } running ({$percent}%)...
operations-running-finished = {$running} {$running ->
    [one] operation
    *[other] operations
  } running ({$percent}%), {$finished} finished...
pause = Pause
resume = Resume

# Dialogs

## Compress Dialog
create-archive = Create archive

## Copy To Dialog
copy-to-title = Select copy destination
copy-to-button-label = Copy

## Extract Dialog
extract-password-required = Password required
extract-to = Extract To...
extract-to-title = Extract to folder

## Empty Trash Dialog
empty-trash = Empty trash
empty-trash-title = Empty trash?
empty-trash-warning = Items in the Trash folder will be permanently deleted

## Mount Error Dialog
mount-error = Unable to access drive

## Move To Dialog
move-to-title = Select move destination
move-to-button-label = Move

## New File/Folder Dialog
create-new-file = Create new file
create-new-folder = Create new folder
file-name = File name
folder-name = Folder name
file-already-exists = A file with that name already exists
folder-already-exists = A folder with that name already exists
name-hidden = Names starting with "." will be hidden
name-invalid = Name cannot be "{$filename}"
name-no-slashes = Name cannot contain slashes

## Open/Save Dialog
cancel = Cancel
create = Create
open = Open
open-file = Open file
open-folder = Open folder
open-in-new-tab = Open in new tab
open-in-new-window = Open in new window
open-item-location = Open item location
open-multiple-files = Open multiple files
open-multiple-folders = Open multiple folders
save = Save
save-file = Save file

## Open With Dialog
open-with-title = How do you want to open "{$name}"?
browse-store = Browse {$store}
other-apps = Other applications
related-apps = Related applications

## Permanently delete Dialog
selected-items = The {$items} selected items
permanently-delete-question = Permanently delete?
delete = Delete
permanently-delete-warning = {$target} will be permanently deleted. This action can't be undone.

## Rename Dialog
rename-file = Rename file
rename-folder = Rename folder

## Replace Dialog
replace = Replace
replace-title = "{$filename}" already exists in this location
replace-warning = Do you want to replace it with the one you are saving? Replacing it will overwrite its content.
replace-warning-operation = Do you want to replace it? Replacing it will overwrite its content.
original-file = Original file
replace-with = Replace with
apply-to-all = Apply to all
keep-both = Keep both
skip = Skip

## Set as Executable and Launch Dialog
set-executable-and-launch = Set as executable and launch
set-executable-and-launch-description = Do you want to set "{$name}" as executable and launch it?
set-and-launch = Set and launch

## Metadata Dialog
open-with = Open with
owner = Owner
group = Group
other = Other
### Mode 0
none = None
### Mode 1 (unusual)
execute-only = Execute-only
### Mode 2 (unusual)
write-only = Write-only
### Mode 3 (unusual)
write-execute = Write and execute
### Mode 4
read-only = Read-only
### Mode 5
read-execute = Read and execute
### Mode 6
read-write = Read and write
### Mode 7
read-write-execute = Read, write, and execute

## Favorite Path Error Dialog
favorite-path-error = Error opening directory
favorite-path-error-description =
    Unable to open "{$path}"
    "{$path}" might not exist or you might not have permission to open it

    Would you like to remove it from the sidebar?
remove = Remove
keep = Keep

# Context Pages

## About
repository = Repository
support = Support

## Add Network Drive
add-network-drive = Add network drive
connect = Connect
connect-anonymously = Connect anonymously
connecting = Connecting...
domain = Domain
enter-server-address = Enter server address
network-drive-description =
    Server addresses include a protocol prefix and address.
    Examples: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Available protocols,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDAV,dav:// or davs://
network-drive-error = Unable to access network drive
password = Password
remember-password = Remember password
try-again = Try again
username = Username

## Operations
cancelled = Cancelled
edit-history = Edit history
history = History
no-history = No items in history.
pending = Pending
progress = {$percent}%
progress-cancelled = {$percent}%, cancelled
progress-failed = {$percent}%, failed
progress-paused = {$percent}%, paused
failed = Failed
complete = Complete
compressing = Compressing {$items} {$items ->
        [one] item
        *[other] items
    } from "{$from}" to "{$to}" ({$progress})...
compressed = Compressed {$items} {$items ->
        [one] item
        *[other] items
    } from "{$from}" to "{$to}"
copy_noun = Copy
pasted-image = Pasted Image
pasted-text = Pasted Text
pasted-video = Pasted Video
creating = Creating "{$name}" in "{$parent}"
created = Created "{$name}" in "{$parent}"
copying = Copying {$items} {$items ->
        [one] item
        *[other] items
    } from "{$from}" to "{$to}" ({$progress})...
copied = Copied {$items} {$items ->
        [one] item
        *[other] items
    } from "{$from}" to "{$to}"
deleting = Deleting {$items} {$items ->
        [one] item
        *[other] items
    } from {trash} ({$progress})...
deleted = Deleted {$items} {$items ->
        [one] item
        *[other] items
    } from {trash}
emptying-trash = Emptying {trash} ({$progress})...
emptied-trash = Emptied {trash}
extracting = Extracting {$items} {$items ->
        [one] item
        *[other] items
    } from "{$from}" to "{$to}" ({$progress})...
extracted = Extracted {$items} {$items ->
        [one] item
        *[other] items
    } from "{$from}" to "{$to}"
setting-executable-and-launching = Setting "{$name}" as executable and launching
set-executable-and-launched = Set "{$name}" as executable and launched
setting-permissions = Setting permissions for "{$name}" to {$mode}
set-permissions = Set permissions for "{$name}" to {$mode}
moving = Moving {$items} {$items ->
        [one] item
        *[other] items
    } from "{$from}" to "{$to}" ({$progress})...
moved = Moved {$items} {$items ->
        [one] item
        *[other] items
    } from "{$from}" to "{$to}"
permanently-deleting = Permanently deleting {$items} {$items ->
        [one] item
        *[other] items
    }
permanently-deleted = Permanently deleted {$items} {$items ->
        [one] item
        *[other] items
    }
removing-from-recents = Removing {$items} {$items ->
        [one] item
        *[other] items
    } from {recents}
removed-from-recents = Removed {$items} {$items ->
        [one] item
        *[other] items
    } from {recents}
renaming = Renaming "{$from}" to "{$to}"
renamed = Renamed "{$from}" to "{$to}"
restoring = Restoring {$items} {$items ->
        [one] item
        *[other] items
    } from {trash} ({$progress})...
restored = Restored {$items} {$items ->
        [one] item
        *[other] items
    } from {trash}
unknown-folder = unknown folder

## Open with
menu-open-with = Open with...
default-app = {$name} (default)

## Show details
show-details = Show details
type = Type: {$mime}
items = Items: {$items}
item-size = Size: {$size}
item-created = Created: {$created}
item-modified = Modified: {$modified}
item-accessed = Accessed: {$accessed}
calculating = Calculating...

## Settings
settings = Settings
single-click = Single click to open

### Appearance
appearance = Appearance
theme = Theme
match-desktop = Match desktop
dark = Dark
light = Light

### Type to search
type-to-search = Type to search
type-to-search-recursive = Searches the current folder and all subfolders
type-to-search-enter-path = Enters the path to the directory or file
type-to-search-select = Selects the first matching file or folder

# Context menu
add-to-sidebar = Add to sidebar
compress = Compress
copy-to = Copy to...
delete-permanently = Delete permanently
eject = Eject
extract-here = Extract
new-file = New file...
new-folder = New folder...
open-in-terminal = Open in terminal
move-to = Move to...
move-to-trash = Move to trash
restore-from-trash = Restore from trash
remove-from-sidebar = Remove from sidebar
sort-by-name = Sort by name
sort-by-modified = Sort by modified
sort-by-size = Sort by size
sort-by-trashed = Sort by delete time
remove-from-recents = Remove from recents

## Desktop
change-wallpaper = Change wallpaper...
desktop-appearance = Desktop appearance...
display-settings = Display settings...

# Menu

## File
file = File
new-tab = New tab
new-window = New window
reload-folder = Reload folder
rename = Rename...
close-tab = Close tab
quit = Quit

## Edit
edit = Edit
cut = Cut
copy = Copy
paste = Paste
select-all = Select all

## View
zoom-in = Zoom in
default-size = Default size
zoom-out = Zoom out
view = View
grid-view = Grid view
list-view = List view
show-hidden-files = Show hidden files
list-directories-first = List directories first
gallery-preview = Gallery preview
menu-settings = Settings...
menu-about = About COSMIC Files...

## Sort
sort = Sort
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Newest first
sort-oldest-first = Oldest first
sort-smallest-to-largest = Smallest to largest
sort-largest-to-smallest = Largest to smallest
