# FaceReg Desktop — Tauri

`https://face-reg-cyan.vercel.app/` saytini Tauri v2 WebView ichida ochadigan
yengil Windows desktop ilova.

## Imkoniyatlar

- faqat native Tauri oynasi va bitta WebView;
- web ilovadagi dizayn bilan bir xil native boshlang‘ich login;
- Tauri login bearer tokenini WebView sessiyasiga avtomatik uzatadi;
- muvaffaqiyatli kirilganda web login sahifasi qayta ko‘rsatilmaydi;
- sayt fayllari ilovaga ko‘chirilmaydi;
- release build'da DevTools o‘chirilgan;
- browser context menu va developer hotkeylar bloklangan;
- `F5` yoki `Ctrl+R` bilan refresh;
- yangi oynada ochiladigan linklar shu WebView ichida ochiladi;
- kamera va mikrofon ruxsatlarini saytning o‘zi boshqaradi.

## GitHub orqali `.exe` build

1. `master` branchga push qiling.
2. **Actions → Build Tauri Windows EXE** workflow'ini oching.
3. Build tugagach **Artifacts** ichidan `FaceReg-Tauri-Windows-x64` ni yuklang.
4. ZIP ichidagi `FaceReg_*_x64-setup.exe` installer'ni ishga tushiring.

Build Windows runner'da bajariladi. Foydalanuvchi kompyuterida Node.js, Rust yoki
.NET kerak emas. Tauri Windows WebView2 Runtime'dan foydalanadi; installer kerak
bo‘lsa WebView2 bootstrapper orqali uni o‘rnatadi.

DevTools foydalanuvchi interfeysi va odatiy hotkeylardan yopilgan. Foydalanuvchi
o‘z kompyuterida ishlayotgan har qanday dastur uchun mutlaq reverse-engineering
himoyasini kafolatlab bo‘lmaydi.
