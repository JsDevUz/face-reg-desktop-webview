using Microsoft.Web.WebView2.Core;
using Microsoft.Web.WebView2.WinForms;

namespace FaceRegDesktop;

internal static class Program
{
    [STAThread]
    private static void Main()
    {
        ApplicationConfiguration.Initialize();
        Application.Run(new MainForm());
    }
}

internal sealed class MainForm : Form
{
    private const string AppUrl = "https://face-reg-cyan.vercel.app/";
    private readonly WebView2 webView = new() { Dock = DockStyle.Fill };

    public MainForm()
    {
        Text = "FaceReg";
        StartPosition = FormStartPosition.CenterScreen;
        WindowState = FormWindowState.Maximized;
        MinimumSize = new Size(800, 600);
        Controls.Add(webView);

        Shown += InitializeWebViewAsync;
        KeyPreview = true;
        KeyDown += HandleFormKeyDown;
    }

    private async void InitializeWebViewAsync(object? sender, EventArgs e)
    {
        try
        {
            await webView.EnsureCoreWebView2Async();

            CoreWebView2Settings settings = webView.CoreWebView2.Settings;
            settings.AreDevToolsEnabled = false;
            settings.AreDefaultContextMenusEnabled = false;
            settings.AreBrowserAcceleratorKeysEnabled = false;
            settings.IsStatusBarEnabled = false;
            settings.IsZoomControlEnabled = false;

            webView.CoreWebView2Controller.AcceleratorKeyPressed += (_, args) =>
            {
                bool keyDown =
                    args.KeyEventKind == CoreWebView2KeyEventKind.KeyDown ||
                    args.KeyEventKind == CoreWebView2KeyEventKind.SystemKeyDown;
                if (!keyDown)
                    return;

                Keys key = (Keys)args.VirtualKey;
                bool control = (Control.ModifierKeys & Keys.Control) != 0;
                bool shift = (Control.ModifierKeys & Keys.Shift) != 0;
                bool refresh = key == Keys.F5 || (control && key == Keys.R);
                bool devTools =
                    key == Keys.F12 ||
                    (control && shift &&
                     (key == Keys.I || key == Keys.J || key == Keys.C));

                if (refresh)
                {
                    BeginInvoke((Action)(() => webView.CoreWebView2.Reload()));
                    args.Handled = true;
                }
                else if (devTools)
                {
                    args.Handled = true;
                }
            };

            webView.CoreWebView2.NewWindowRequested += (_, args) =>
            {
                args.Handled = true;
                webView.CoreWebView2.Navigate(args.Uri);
            };

            webView.CoreWebView2.PermissionRequested += (_, args) =>
            {
                // The website remains responsible for asking the user for camera
                // and microphone permission. WebView2 persists the user's choice.
                args.State = CoreWebView2PermissionState.Default;
            };

            webView.CoreWebView2.Navigate(AppUrl);
        }
        catch (WebView2RuntimeNotFoundException)
        {
            MessageBox.Show(
                "Microsoft Edge WebView2 Runtime topilmadi. Uni o‘rnating va ilovani qayta oching.",
                "FaceReg",
                MessageBoxButtons.OK,
                MessageBoxIcon.Error);
            Close();
        }
        catch (Exception ex)
        {
            MessageBox.Show(
                $"Saytni ochib bo‘lmadi.\n\n{ex.Message}",
                "FaceReg",
                MessageBoxButtons.OK,
                MessageBoxIcon.Error);
        }
    }

    private void HandleFormKeyDown(object? sender, KeyEventArgs e)
    {
        bool refresh = e.KeyCode == Keys.F5 || (e.Control && e.KeyCode == Keys.R);
        bool devTools =
            e.KeyCode == Keys.F12 ||
            (e.Control && e.Shift &&
             (e.KeyCode == Keys.I || e.KeyCode == Keys.J || e.KeyCode == Keys.C));

        if (refresh && webView.CoreWebView2 is not null)
        {
            webView.CoreWebView2.Reload();
            e.Handled = true;
            e.SuppressKeyPress = true;
        }
        else if (devTools)
        {
            e.Handled = true;
            e.SuppressKeyPress = true;
        }
    }
}
