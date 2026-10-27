
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::{
        ClientToScreen, GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
    },
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameWindow {
    pub hwnd: usize,
    pub pid: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
struct Search<'a> {
    pids: &'a [u32],
    found: Option<GameWindow>,
    area: i64,
}
unsafe extern "system" fn enumerate(hwnd: HWND, data: LPARAM) -> i32 {
    let search = unsafe { &mut *(data as *mut Search<'_>) };
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if !search.pids.contains(&pid)
        || unsafe { IsWindowVisible(hwnd) } == 0
        || unsafe { GetWindow(hwnd, GW_OWNER) } != std::ptr::null_mut()
    {
        return 1;
    }
    let mut class = [0u16; 80];
    let len = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), 80) };
    if String::from_utf16_lossy(&class[..len.max(0) as usize]) == "ConsoleWindowClass" {
        return 1;
    }
    let target = GameWindow {
        hwnd: hwnd as usize,
        pid,
    };
    if let Some(rect) = bounds(target) {
        let area = if foreground() == target.hwnd {
            i64::MAX
        } else {
            rect.width as i64 * rect.height as i64
        };
        if rect.width >= 240 && rect.height >= 180 && area > search.area {
            search.found = Some(target);
            search.area = area;
        }
    }
    1
}
pub fn find(pids: &[u32]) -> Option<GameWindow> {
    let mut search = Search {
        pids,
        found: None,
        area: 0,
    };
    unsafe {
        EnumWindows(Some(enumerate), &mut search as *mut _ as isize);
    }
    search.found
}
pub fn valid(target: GameWindow) -> bool {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(target.hwnd as HWND, &mut pid);
        IsWindow(target.hwnd as HWND) != 0 && pid == target.pid
    }
}
pub fn bounds(target: GameWindow) -> Option<Bounds> {
    if !valid(target) || unsafe { IsIconic(target.hwnd as HWND) } != 0 {
        return None;
    }
    let mut rect = RECT::default();
    let mut point = POINT::default();
    if unsafe { GetClientRect(target.hwnd as HWND, &mut rect) } == 0
        || unsafe { ClientToScreen(target.hwnd as HWND, &mut point) } == 0
    {
        return None;
    }
    let result = Bounds {
        x: point.x,
        y: point.y,
        width: rect.right,
        height: rect.bottom,
    };
    (result.width > 0 && result.height > 0).then_some(result)
}
pub fn foreground() -> usize {
    unsafe { GetForegroundWindow() as usize }
}
pub fn key(code: i32) -> bool {
    unsafe { GetAsyncKeyState(code) < 0 }
}
pub fn fit(overlay: usize, rect: Bounds) {
    unsafe {
        SetWindowPos(
            overlay as HWND,
            HWND_TOPMOST,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS,
        );
    }
}
pub fn focus(hwnd: usize) -> bool {
    use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    unsafe {
        if IsWindow(hwnd as HWND) == 0 {
            return false;
        }
        if SetForegroundWindow(hwnd as HWND) != 0 {
            return true;
        }
        let foreground = GetForegroundWindow();
        if foreground.is_null() || IsHungAppWindow(foreground) != 0 {
            return false;
        }
        let source = GetCurrentThreadId();
        let destination = GetWindowThreadProcessId(foreground, std::ptr::null_mut());
        if source == destination || AttachThreadInput(source, destination, 1) == 0 {
            return false;
        }
        let result = SetForegroundWindow(hwnd as HWND) != 0;
        AttachThreadInput(source, destination, 0);
        result
    }
}
pub fn mouse() -> (bool, bool) {
    if unsafe { GetSystemMetrics(SM_SWAPBUTTON) } != 0 {
        (key(2), key(1))
    } else {
        (key(1), key(2))
    }
}
pub fn release_cursor() {
    unsafe {
        ClipCursor(std::ptr::null());
    }
}

pub fn debug_shortcut(target:GameWindow,letter:u16)->Result<(),String>{
    if !valid(target)||foreground()!=target.hwnd||![0x42,0x47,0x44,0x48,0x54].contains(&letter){return Err("Minecraft debe tener el foco para recibir el atajo".into());}
    if key(VK_F3 as i32)||key(VK_CONTROL as i32)||key(VK_MENU as i32){return Err("Suelta F3, Control y Alt antes de usar este control".into());}
    let input=|code:u16,up:bool|INPUT{r#type:INPUT_KEYBOARD,Anonymous:INPUT_0{ki:KEYBDINPUT{wVk:code,wScan:0,dwFlags:if up{KEYEVENTF_KEYUP}else{0},time:0,dwExtraInfo:0}}};
    let events=[input(VK_F3,false),input(letter,false),input(letter,true),input(VK_F3,true)];
    let count=unsafe{SendInput(events.len()as u32,events.as_ptr(),std::mem::size_of::<INPUT>()as i32)};
    if count!=events.len()as u32{return Err("Windows no permitió enviar el atajo al juego".into());}Ok(())
}

pub struct HudHotkey {
    registered: bool,
}
impl HudHotkey {
    pub fn new() -> Self {
        Self {
            registered: unsafe { RegisterHotKey(std::ptr::null_mut(), 0x5450, MOD_NOREPEAT, 0x77) }
                != 0,
        }
    }
    pub fn registered(&self) -> bool {
        self.registered
    }
    pub fn pressed(&self) -> bool {
        let mut message = MSG::default();
        let mut pressed = false;
        while unsafe {
            PeekMessageW(
                &mut message,
                std::ptr::null_mut(),
                WM_HOTKEY,
                WM_HOTKEY,
                PM_REMOVE,
            )
        } != 0
        {
            if message.wParam == 0x5450 {
                pressed = true;
            }
        }
        pressed
    }
}
impl Drop for HudHotkey {
    fn drop(&mut self) {
        if self.registered {
            unsafe {
                UnregisterHotKey(std::ptr::null_mut(), 0x5450);
            }
        }
    }
}
pub struct Borderless {
    target: GameWindow,
    style: isize,
    placement: WINDOWPLACEMENT,
    visible: bool,
}
impl Borderless {
    pub fn apply(target: GameWindow) -> Option<Self> {
        if !valid(target) {
            return None;
        }
        let handle = target.hwnd as HWND;
        let style = unsafe { GetWindowLongPtrW(handle, GWL_STYLE) };
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        if unsafe { GetWindowPlacement(handle, &mut placement) } == 0 {
            return None;
        }
        let mut monitor = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if unsafe {
            GetMonitorInfoW(
                MonitorFromWindow(handle, MONITOR_DEFAULTTONEAREST),
                &mut monitor,
            )
        } == 0
        {
            return None;
        }
        let r = monitor.rcMonitor;
        unsafe {
            SetWindowLongPtrW(
                handle,
                GWL_STYLE,
                (style & !(WS_OVERLAPPEDWINDOW as isize)) | WS_POPUP as isize,
            );
            SetWindowPos(
                handle,
                std::ptr::null_mut(),
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_ASYNCWINDOWPOS,
            );
        }
        Some(Self {
            target,
            style,
            placement,
            visible: unsafe { IsWindowVisible(handle) } != 0,
        })
    }
    pub fn target(&self) -> GameWindow {
        self.target
    }


    pub fn maintain(&self){
        if !valid(self.target){return;}
        unsafe{let handle=self.target.hwnd as HWND;let mut monitor=MONITORINFO{cbSize:std::mem::size_of::<MONITORINFO>() as u32,..Default::default()};
        if GetMonitorInfoW(MonitorFromWindow(handle,MONITOR_DEFAULTTONEAREST),&mut monitor)==0{return;}
        let r=monitor.rcMonitor;let mut actual=RECT::default();if GetWindowRect(handle,&mut actual)==0{return;}
        let style=GetWindowLongPtrW(handle,GWL_STYLE);if style&WS_OVERLAPPEDWINDOW as isize!=0||actual.left!=r.left||actual.top!=r.top||actual.right!=r.right||actual.bottom!=r.bottom{
        SetWindowLongPtrW(handle,GWL_STYLE,(style&!(WS_OVERLAPPEDWINDOW as isize))|WS_POPUP as isize);SetWindowPos(handle,std::ptr::null_mut(),r.left,r.top,r.right-r.left,r.bottom-r.top,SWP_NOZORDER|SWP_NOACTIVATE|SWP_FRAMECHANGED|SWP_ASYNCWINDOWPOS);}}
    }
}
impl Drop for Borderless {
    fn drop(&mut self) {
        if valid(self.target) {
            unsafe {
                let handle = self.target.hwnd as HWND;
                SetWindowLongPtrW(handle, GWL_STYLE, self.style);
                SetWindowPlacement(handle, &self.placement);
                if !self.visible {
                    ShowWindow(handle, SW_HIDE);
                }
                SetWindowPos(
                    handle,
                    std::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE
                        | SWP_NOSIZE
                        | SWP_NOZORDER
                        | SWP_NOACTIVATE
                        | SWP_FRAMECHANGED
                        | SWP_ASYNCWINDOWPOS,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borderless_restores_hidden_window_bounds_and_style() {
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let title: Vec<u16> = "TP HUD geometry test\0".encode_utf16().collect();
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_NOACTIVATE,
                class.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                80,
                90,
                640,
                480,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        assert!(
            !hwnd.is_null(),
            "No se pudo crear la ventana de prueba Win32"
        );
        struct Cleanup(HWND);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                unsafe {
                    DestroyWindow(self.0);
                }
            }
        }
        let _cleanup = Cleanup(hwnd);
        let target = GameWindow {
            hwnd: hwnd as usize,
            pid: std::process::id(),
        };
        assert!(valid(target));
        assert!(!valid(GameWindow { pid: 0, ..target }));
        let original = bounds(target).unwrap();
        let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };
        let borderless =
            Borderless::apply(target).expect("Sin bordes debe funcionar para una ventana propia");
        assert_eq!(
            unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } & (WS_CAPTION as isize),
            0
        );
        assert!(bounds(target).unwrap().width >= original.width);
        drop(borderless);
        assert_eq!(unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) }, style);
        assert_eq!(bounds(target), Some(original));
    }
}
