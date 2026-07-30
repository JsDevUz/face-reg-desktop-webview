# Xodimlar tizimi — Tauri

`https://face-reg-cyan.vercel.app/` saytini Tauri v2 WebView ichida ochadigan
yengil Windows desktop ilova.

## Muhit rejimi

Repo ildizidagi `.env` faylda faqat bittasini tanlang:

```env
MODE=DEV
```

yoki:

```env
MODE=PROD
```

- `DEV` barcha asosiy API requestlarni `https://api.tayin.uz` ga yuboradi,
  EPOS uchun `https://api.tayin.uz/v1/helper/epos` ishlatadi va tayin mock
  javobini qabul qiladi.
- `PROD` barcha asosiy API requestlarni
  `https://api.pharma-cosmos.uz:4443` ga yuboradi, EPOS uchun
  `http://localhost:8347/uzpos` ishlatadi va terminal ID qat’iy mos kelishini
  talab qiladi.

## Imkoniyatlar

- faqat native Tauri oynasi va bitta WebView;
- web ilovadagi dizayn bilan bir xil native boshlang‘ich login;
- Tauri login bearer tokenini WebView sessiyasiga avtomatik uzatadi;
- muvaffaqiyatli kirilganda web login sahifasi qayta ko‘rsatilmaydi;
- WebView ochilishidan oldin EPOS terminal ID dorixona terminal ID’lari bilan
  native Rust qatlamida solishtiriladi;
- terminal ID dorixonaga biriktirilgan ID bilan to‘liq mos kelmasa, ID
  topilmasa yoki lokal EPOS servisiga ulanib bo‘lmasa web ochilmaydi va native
  “Kirish bloklandi” oynasi ko‘rsatiladi;
- sayt fayllari ilovaga ko‘chirilmaydi;
- release build'da DevTools o‘chirilgan;
- browser context menu va developer hotkeylar bloklangan;
- `F5` yoki `Ctrl+R` bilan refresh;
- yangi oynada ochiladigan linklar shu WebView ichida ochiladi;
- kamera va mikrofon ruxsatlarini saytning o‘zi boshqaradi.

## GitHub orqali `.exe` build

1. `master` branchga push qiling.
2. **Actions → Build Tauri Windows EXE** workflow'ini oching.
3. Build tugagach **Artifacts** ichidan `Xodimlar-tizimi-Windows-x64` ni yuklang.
4. ZIP ichidagi `Xodimlar tizimi_*_x64-setup.exe` installer'ni ishga tushiring.

Build Windows runner'da bajariladi. Foydalanuvchi kompyuterida Node.js, Rust yoki
.NET kerak emas. Tauri Windows WebView2 Runtime'dan foydalanadi; installer kerak
bo‘lsa WebView2 bootstrapper orqali uni o‘rnatadi.

DevTools foydalanuvchi interfeysi va odatiy hotkeylardan yopilgan. Foydalanuvchi
o‘z kompyuterida ishlayotgan har qanday dastur uchun mutlaq reverse-engineering
himoyasini kafolatlab bo‘lmaydi.
