# room-model

Crate thuần (ADR 0016) cho phòng học 3D của trang chủ: bảng chỗ đặt đồ `ROOM`, khu `Zone` và thông điệp cầu nối `ToRoom` / `FromRoom` (JSON, `{"t":"Go","v":"Review"}`). `web` và `room3d` cùng đọc bảng này nên "vật nào dẫn vào khu nào" chỉ có một nguồn. Không biết Bevy, Leptos hay nội dung học. Vị trí tính bằng cm, số nguyên, để tất định.
