cosmic-files = ตัวจัดการไฟล์ COSMIC
empty-folder = แฟ้มเปล่า
empty-folder-hidden = แฟ้มเปล่า (มีแฟ้มที่ซ่อนอยู่)
no-results = ไม่พบผลลัพธ์
filesystem = ระบบไฟล์
home = บ้าน
networks = เครือข่าย
notification-in-progress = กำลังดำเนินการไฟล์
trash = ถังขยะ
recents = ล่าสุด
undo = เลิกทำ
today = วันนี้
# Desktop view options
desktop-view-options = ตัวเลือกมุมมองหน้าจอหลัก...
show-on-desktop = แสดงบนหน้าจอหลัก
desktop-folder-content = เนื้อหาแฟ้มหน้าจอหลัก
mounted-drives = ไดร์ฟที่ใช้งานได้
trash-folder-icon = ไอคอนแฟ้มถังขยะ
icon-size-and-spacing = ขนาดและระยะห่างไอคอน
icon-size = ขนาดไอคอน
# List view
name = ชื่อ
modified = แก้ไขล่าสุด
trashed-on = ถูกทิ้ง
size = ขนาด
# Progress footer
details = รายละเอียด
dismiss = ไม่สนใจข้อความ
operations-running = การดำเนินการ { $running } running ({ $percent }%)...
operations-running-finished = { $running } operations running ({ $percent }%), { $finished } finished...
pause = หยุด
resume = ทำต่อ

# Dialogs


## Compress Dialog

create-archive = สร้างไฟล์บีบอัด

## Empty Trash Dialog

empty-trash = ล้างถังขยะ
empty-trash-warning = คุณแน่ใจหรือไม่ว่าคุณต้องการจะลบภายในถังขยะถาวร

## Mount Error Dialog

mount-error = ไม่สามารถเข้าถึงไดร์ฟได้

## New File/Folder Dialog

create-new-file = สร้างไฟล์ใหม่
create-new-folder = สร้างแฟ้มใหม่
file-name = ชื่อไฟล์
folder-name = ชื่อแฟ้ม
file-already-exists = มีไฟล์ชื่อนี้อยู่แล้ว
folder-already-exists = มีแฟ้มชื่อนี้อยู่แล้ว
name-hidden = ชื่อที่ขึ้นต้นด้วย "." จะถูกซ่อน
name-invalid = ไม่สามารถตั้ง "{ $filename }" เป็นชื่อได้
name-no-slashes = ชื่อไม่สามารถมีเครื่องหมายทับได้

## Open/Save Dialog

cancel = ยกเลิก
create = สร้าง
open = เปิด
open-file = เปิดไฟล์
open-folder = เปิดแฟ้ม
open-in-new-tab = เปิดในแทบใหม่
open-in-new-window = เปิดในหน้าต่างใหม่
open-item-location = เปิดตำแหน่งของรายการ
open-multiple-files = เปิดหลายไฟล์
open-multiple-folders = เปิดหลายแฟ้ม
save = บันทึก
save-file = บักทึกไฟล์

## Open With Dialog

open-with-title = คุณจะเปิดไฟล์ "{ $name }" อย่างไร
browse-store = เรียกดูใน { $store }

## Rename Dialog

rename-file = เปลี่ยนชื่อไฟล์
rename-folder = เปลี่ยนชื่อแฟ้ม

## Replace Dialog

replace = แทนที่
replace-title = มี "{ $filename }" อยู่แล้วที่ตำแหน่งนี้
replace-warning = คุณต้องการจะแทนที่ไฟล์ด้วยไฟล์ที่คุณกำลังบันทึกอยู่หรือไม่ การแทนที่จะเขียนทับเนื้อหาเดิม
replace-warning-operation = คุณต้องการจะแทนที่ไฟล์หรือไม่ การแทนที่จะเขียนทับเนื้อหาเดิม
original-file = ไฟล์ต้นฉบับ
replace-with = แทนที่ด้วย
apply-to-all = นำไปใช้กับทั้งหมด
keep-both = เก็บไว้ทั้งคู่
skip = ข้าม

## Set as Executable and Launch Dialog

set-executable-and-launch = ตั้งเป็นไฟล์ที่สามารถรันได้และเปิด
set-executable-and-launch-description = คุณต้องการที่จะตั้งไฟล์ "{ $name }" ให้สามารถรันได้และเปิดเลยหรือไม่
set-and-launch = ตั้งและเปิด

## Metadata Dialog

owner = เจ้าของ
group = กลุ่ม
other = ผู้อื่น

# Context Pages


## About


## Add Network Drive

add-network-drive = เพิ่มไดรฟ์เครือข่าย
connect = เชื่อมต่อ
connect-anonymously = เชื่อมต่อแบบไม่ระบุตัวตน
connecting = กำลังเชื่อมต่อ...
domain = โดเมน
enter-server-address = ใส่ที่อยู่เซิร์ฟเวอร์
network-drive-description =
    ที่อยู่ของเซิร์ฟเวอร์ประกอบด้วยโปรโตคอลและที่อยู่
    เช่น: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    โปรโตคอลที่ใช้งานได้,โปรโตคอล
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = ไม่สามารถเข้าถึงไดร์ฟเครือข่าย
password = รหัสผ่าน
remember-password = จดจำรหัสผ่าน
try-again = ลองอีกครั้ง
username = ชื่อผู้ใช้

## Operations

cancelled = ยกเลิกแล้ว
edit-history = แก้ไขประวัติ
history = ประวัติ
no-history = ไม่มีไฟล์ในประวัติ
pending = รอดำเนินการ
progress = { $percent }%
progress-cancelled = { $percent }%, ยกเลิกแล้ว
progress-paused = { $percent }%, หยุดชั่วคราว
failed = ล้มเหลว
complete = เสร็จสิ้น
compressing =
    กำลังบีบอัด { $items } { $items ->
        [one] ไฟล์
       *[other] ไฟล์
    } จาก "{ $from }" สู่ "{ $to }" ({ $progress })...
compressed =
    บีบอัด { $items } { $items ->
        [one] ไฟล์
       *[other] ไฟล์
    } จาก "{ $from }" สู่ "{ $to }"
copy_noun = คัดลอก
creating = กำลังสร้างไฟล์ "{ $name }" ใน "{ $parent }
created = สร้าง "{ $name }" ใน "{ $parent }" แล้ว
copying =
    กำลังคำลอก { $items } { $items ->
        [one] ไฟล์
       *[other] ไฟล์
    } จาก "{ $from }" สู่ "{ $to }" ({ $progress })...
copied =
    เสร็จสิ้นการคัดลอก { $items } { $items ->
        [one] ไฟล์
       *[other] ไฟล์
    } จาก "{ $from }" สู่ "{ $to }"
emptying-trash = กำลังล้างถังขยะ { trash } ไฟล์ ({ $progress })...
emptied-trash = ล้างถังขยะแล้ว { trash } ไฟล์
extracting =
    กำลังแตกไฟล์ { $items } { $items ->
        [one] ไฟล์
       *[other] ไฟล์
    } จาก "{ $from }" สู่ "{ $to }" ({ $progress })...
extracted =
    เสร็จสิ้นการแตกไฟล์ { $items } { $items ->
        [one] ไฟล์
       *[other] ไฟล์
    } จาก "{ $from }" สู่ "{ $to }"
setting-executable-and-launching = กำลังตั้งไฟล์ "{ $name }" ให้สามารถรันได้และเปิด
set-executable-and-launched = ตั้งไฟล์ "{ $name }" ให้สามารถรันได้และเปิดแล้ว
moving =
    กำลังย้ายไฟล์ { $items } { $items ->
        [one] ไฟล์
       *[other] ไฟล์
    } จาก "{ $from }" สู่ "{ $to }" ({ $progress })...
moved =
    เสร็จสิ้นการย้ายไฟล์ { $items } { $items ->
        [one] ไฟล์
       *[other] ไฟล์
    } จาก "{ $from }" สู่ "{ $to }"
renaming = กำลังเปลี่ยนชื่อจาก "{ $from }" เป็น "{ $to }"
renamed = Renamed "{ $from }" to "{ $to }"
restoring =
    Restoring { $items } { $items ->
        [one] item
       *[other] items
    } from { trash } ({ $progress })...
restored =
    Restored { $items } { $items ->
        [one] item
       *[other] items
    } from { trash }
unknown-folder = แฟ้มที่ไม่รู้จัก

## Open with

menu-open-with = เปิดด้วย...
default-app = { $name } (ค่าเริ่มต้น)

## Show details

show-details = แสดงรายละเอียด
type = ชนิด: { $mime }
items = ไฟล์: { $items }
item-size = ขนาดไฟล์: { $size }
item-created = สร้างเมื่อ: { $created }
item-modified = แก้ไขเมื่อ: { $modified }
item-accessed = เปิดใช้เมื่อ: { $accessed }
calculating = กำลังคำนวณ...

## Settings

settings = การตั้งค่า

### Appearance

appearance = ลักษณะ
theme = ธีม
match-desktop = ใช้ตามธีมหน้าจอหลัก
dark = ธีมมืด
light = ธีมสว่าง
# Context menu
add-to-sidebar = เพิ่มเข้าแถบด้านข้าง
compress = บีบอัด
extract-here = แตกไฟล์
new-file = สร้างไฟล์...
new-folder = สร้างแฟ้ม...
open-in-terminal = เปิดในเทอร์มินัล
move-to-trash = ย้ายไปถังขยะ
restore-from-trash = เรียกคืนจากถังขยะ
remove-from-sidebar = นำออกจากแถบด้านข้าง
sort-by-name = เรียงตามชื่อ
sort-by-modified = เรียงตามเวลาแก้ไขล่าสุด
sort-by-size = เรียงตามขนาด
sort-by-trashed = เรียงตามเวลาลบ

## Desktop

change-wallpaper = เปลี่ยนภาพพื้นหลัง...
desktop-appearance = ลักษณะหน้าจอหลัก...
display-settings = การตั้งค่าหน้าจอแสดงผล...

# Menu


## File

file = ไฟล์
new-tab = แทบใหม่
new-window = หน้าต่างใหม่
rename = เปลี่ยนชื่อ...
close-tab = ปิดแทบ
quit = ออก

## Edit

edit = แก้ไข
cut = ตัด
copy = คัดลอก
paste = วาง
select-all = เลือกทั้งหมด

## View

zoom-in = ซูมเข้า
default-size = ขนาดดั้งเดิม
zoom-out = ซูมออก
view = มุมมอง
grid-view = มุมมองแบบตาราง
list-view = มุมมองแบบรายการ
show-hidden-files = แสดงไฟล์ที่ซ่อนอยู่
list-directories-first = แสดงแฟ้มก่อนเสมอ
gallery-preview = ตัวอย่างแบบแกลเลอรี่
menu-settings = การตั้งค่า...
menu-about = เกี่ยวกับตัวจัดการไฟล์ COSMIC...

## Sort

sort = เรียง
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = ไฟล์ใหม่ก่อน
sort-oldest-first = ไฟล์เก่าก่อน
sort-smallest-to-largest = ขนาดเล็กก่อน
sort-largest-to-smallest = ขนาดใหญ่ก่อน
