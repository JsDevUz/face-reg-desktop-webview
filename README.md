# FaceReg Desktop

`https://face-reg-cyan.vercel.app/` saytini Windows WebView2 ichida ochadigan
yengil desktop ilova.

## Imkoniyatlar

- faqat bitta WebView oynasi;
- DevTools, browser context menu va developer hotkeylar bloklangan;
- `F5` yoki `Ctrl+R` bilan refresh;
- yangi oynada ochiladigan linklar shu oynaning o‘zida ochiladi;
- sayt fayllari kompyuterga bundle qilinmaydi.

## GitHub orqali EXE build

1. Repoga commit qilib GitHub'ga push qiling.
2. GitHub'da **Actions → Build Windows EXE → Run workflow** ni bosing.
3. Build tugagach, **Artifacts** bo‘limidan `FaceReg-Windows-x64` ni yuklab oling.
4. ZIP ichidagi fayllarni bitta papkada saqlab, `FaceReg.exe` ni oching.

Build framework-dependent qilingan: natija kichik bo‘ladi, lekin Windows
kompyuterda **.NET 8 Desktop Runtime (x64)** va **Microsoft Edge WebView2
Runtime** bo‘lishi kerak. Zamonaviy Windows 10/11 tizimlarida WebView2 odatda
allaqachon o‘rnatilgan bo‘ladi.

## Xavfsizlik izohi

DevTools foydalanuvchi interfeysidan yopilgan. Bu odatiy foydalanuvchining
inspector ochishini to‘xtatadi, ammo foydalanuvchining o‘z kompyuterida mutlaq
teskari tahlil himoyasini kafolatlab bo‘lmaydi.
