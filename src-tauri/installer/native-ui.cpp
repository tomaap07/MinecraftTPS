#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <commctrl.h>
#include <dwmapi.h>
#include <uxtheme.h>
#include <richedit.h>
#include <map>
#include <string>
#include <vector>

static constexpr COLORREF Background=RGB(16,12,25),Surface=RGB(27,19,39),Accent=RGB(168,85,247),Text=RGB(239,229,250),Muted=RGB(185,165,207);
static HBRUSH backgroundBrush=CreateSolidBrush(Background),surfaceBrush=CreateSolidBrush(Surface);
static HWND mainWindow=nullptr;
static HFONT uiFont=nullptr,headingFont=nullptr;
static int titleHeight=36,captionButtonWidth=45;
static constexpr int MinimizeId=0x7f01,MaximizeId=0x7f02,CloseId=0x7f03;
struct Layout {RECT rect;SIZE parent;};
static std::map<HWND,Layout> layouts;
static std::map<HWND,bool> hovered;

static std::wstring className(HWND window){wchar_t name[128]{};GetClassNameW(window,name,128);return name;}
static std::wstring textOf(HWND window){int length=GetWindowTextLengthW(window);std::wstring text(length+1,L'\0');GetWindowTextW(window,text.data(),length+1);text.resize(length);return text;}
static void fill(HDC dc,const RECT& rect,COLORREF color){HBRUSH brush=CreateSolidBrush(color);FillRect(dc,&rect,brush);DeleteObject(brush);}
static void line(HDC dc,int x1,int y1,int x2,int y2,COLORREF color,int width=1){HPEN pen=CreatePen(PS_SOLID,width,color);auto old=SelectObject(dc,pen);MoveToEx(dc,x1,y1,nullptr);LineTo(dc,x2,y2);SelectObject(dc,old);DeleteObject(pen);}

static void drawScrollbars(HWND window){
    RECT bounds{};if(!GetWindowRect(window,&bounds))return;HDC dc=GetWindowDC(window);if(!dc)return;
    for(LONG object:{OBJID_VSCROLL,OBJID_HSCROLL}){
        SCROLLBARINFO info{};info.cbSize=sizeof(info);
        if(!GetScrollBarInfo(window,object,&info)||(info.rgstate[0]&(STATE_SYSTEM_INVISIBLE|STATE_SYSTEM_OFFSCREEN)))continue;
        RECT bar=info.rcScrollBar;OffsetRect(&bar,-bounds.left,-bounds.top);fill(dc,bar,Surface);
        bool vertical=object==OBJID_VSCROLL;int center=vertical?(bar.left+bar.right)/2:(bar.top+bar.bottom)/2;
        int first=(vertical?bar.top:bar.left)+info.dxyLineButton/2,last=(vertical?bar.bottom:bar.right)-info.dxyLineButton/2;
        if(vertical){line(dc,center-3,first+1,center,first-2,Muted);line(dc,center,first-2,center+3,first+1,Muted);line(dc,center-3,last-1,center,last+2,Muted);line(dc,center,last+2,center+3,last-1,Muted);}
        else{line(dc,first+1,center-3,first-2,center,Muted);line(dc,first-2,center,first+1,center+3,Muted);line(dc,last-1,center-3,last+2,center,Muted);line(dc,last+2,center,last-1,center+3,Muted);}
        if(info.xyThumbBottom>info.xyThumbTop){RECT thumb=bar;if(vertical){thumb.top=bar.top+info.xyThumbTop;thumb.bottom=bar.top+info.xyThumbBottom;thumb.left+=3;thumb.right-=3;}else{thumb.left=bar.left+info.xyThumbTop;thumb.right=bar.left+info.xyThumbBottom;thumb.top+=3;thumb.bottom-=3;}fill(dc,thumb,Accent);}
    }
    ReleaseDC(window,dc);
}

static void drawButton(HWND window,HDC dc){
    RECT rect{};GetClientRect(window,&rect);const int id=GetDlgCtrlID(window);bool enabled=IsWindowEnabled(window)!=FALSE;
    LRESULT state=SendMessageW(window,BM_GETSTATE,0,0);bool hot=hovered[window],pressed=(state&BST_PUSHED)!=0;
    const LONG_PTR type=GetWindowLongPtrW(window,GWL_STYLE)&BS_TYPEMASK;
    const bool check=type==BS_CHECKBOX||type==BS_AUTOCHECKBOX||type==BS_3STATE||type==BS_AUTO3STATE;
    const bool radio=type==BS_RADIOBUTTON||type==BS_AUTORADIOBUTTON;
    fill(dc,rect,check||radio?Background:pressed?RGB(84,43,116):id==1&&enabled?RGB(123,51,181):hot?RGB(65,37,89):Surface);
    SetBkMode(dc,TRANSPARENT);SetTextColor(dc,enabled?Text:RGB(107,94,126));auto oldFont=SelectObject(dc,uiFont);
    if(id>=MinimizeId&&id<=CloseId){
        if(id==CloseId&&hot)fill(dc,rect,RGB(122,36,80));
        int x=(rect.right-rect.left)/2,y=(rect.bottom-rect.top)/2;
        if(id==MinimizeId)line(dc,x-5,y+1,x+5,y+1,Text);
        if(id==MaximizeId){RECT box{x-5,y-5,x+5,y+5};HBRUSH clear=(HBRUSH)GetStockObject(HOLLOW_BRUSH);HPEN pen=CreatePen(PS_SOLID,1,Text);auto oldPen=SelectObject(dc,pen),oldBrush=SelectObject(dc,clear);Rectangle(dc,box.left,box.top,box.right,box.bottom);SelectObject(dc,oldPen);SelectObject(dc,oldBrush);DeleteObject(pen);}
        if(id==CloseId){line(dc,x-5,y-5,x+5,y+5,Text);line(dc,x+5,y-5,x-5,y+5,Text);}
    }else if(type==BS_GROUPBOX){
        RECT box=rect;box.top+=8;HBRUSH clear=(HBRUSH)GetStockObject(HOLLOW_BRUSH);HPEN pen=CreatePen(PS_SOLID,1,RGB(66,43,85));auto oldPen=SelectObject(dc,pen),oldBrush=SelectObject(dc,clear);Rectangle(dc,box.left,box.top,box.right,box.bottom);SelectObject(dc,oldPen);SelectObject(dc,oldBrush);DeleteObject(pen);rect.left+=12;rect.bottom=rect.top+24;auto text=textOf(window);DrawTextW(dc,text.c_str(),-1,&rect,DT_LEFT|DT_SINGLELINE);
    }else{
        if(check||radio){
            int side=14,top=(rect.bottom-side)/2;RECT box{2,top,2+side,top+side};fill(dc,box,RGB(29,20,42));FrameRect(dc,&box,enabled?surfaceBrush:backgroundBrush);
            if(SendMessageW(window,BM_GETCHECK,0,0)!=BST_UNCHECKED){fill(dc,box,Accent);line(dc,4,top+7,8,top+11,Text,2);line(dc,8,top+11,14,top+3,Text,2);}
            rect.left=side+11;
        }else{RECT border=rect;HBRUSH brush=CreateSolidBrush(hot?Accent:RGB(65,44,85));FrameRect(dc,&border,brush);DeleteObject(brush);rect.left+=8;rect.right-=8;}
        auto text=textOf(window);DrawTextW(dc,text.c_str(),-1,&rect,(check||radio?DT_LEFT:DT_CENTER)|DT_VCENTER|DT_SINGLELINE|DT_END_ELLIPSIS);
    }
    if(GetFocus()==window&&enabled){RECT focus=rect;InflateRect(&focus,-3,-3);HBRUSH brush=CreateSolidBrush(Accent);FrameRect(dc,&focus,brush);DeleteObject(brush);}
    SelectObject(dc,oldFont);
}

static LRESULT CALLBACK controlProc(HWND window,UINT message,WPARAM wParam,LPARAM lParam,UINT_PTR,DWORD_PTR){
    auto name=className(window);
    if(name==L"Button"&&(message==WM_PAINT||message==WM_PRINTCLIENT)){
        if(message==WM_PAINT){PAINTSTRUCT paint{};HDC dc=BeginPaint(window,&paint);drawButton(window,dc);EndPaint(window,&paint);return 0;}
        drawButton(window,(HDC)wParam);return 0;
    }
    if(name==L"Button"&&message==WM_ERASEBKGND)return 1;
    if(name==L"Button"&&message==WM_MOUSEMOVE){hovered[window]=true;TRACKMOUSEEVENT track{sizeof(track),TME_LEAVE,window,0};TrackMouseEvent(&track);InvalidateRect(window,nullptr,FALSE);}
    if(name==L"Button"&&message==WM_MOUSELEAVE){hovered[window]=false;InvalidateRect(window,nullptr,FALSE);}
    if(message==WM_NCDESTROY){layouts.erase(window);hovered.erase(window);RemoveWindowSubclass(window,controlProc,1);}
    LRESULT result=DefSubclassProc(window,message,wParam,lParam);
    if(name==L"Button"&&(message==WM_ENABLE||message==WM_SETTEXT||message==WM_SETFOCUS||message==WM_KILLFOCUS||message==WM_LBUTTONDOWN||message==WM_LBUTTONUP||message==BM_SETCHECK))InvalidateRect(window,nullptr,FALSE);
    if(name!=L"Button"&&(message==WM_NCPAINT||message==WM_PAINT||message==WM_VSCROLL||message==WM_HSCROLL||message==WM_NCMOUSEMOVE||message==WM_NCLBUTTONUP||message==WM_MOUSEWHEEL||message==WM_KEYUP||message==WM_SIZE))drawScrollbars(window);
    return result;
}

static void resizeChildren(HWND parent){
    RECT area{};GetClientRect(parent,&area);
    for(HWND child=GetWindow(parent,GW_CHILD);child;child=GetWindow(child,GW_HWNDNEXT)){
        int id=GetDlgCtrlID(child);if(parent==mainWindow&&id>=MinimizeId&&id<=CloseId)continue;
        auto found=layouts.find(child);if(found==layouts.end()||found->second.parent.cx<=0||found->second.parent.cy<=0)continue;
        const auto &base=found->second;double x=double(area.right)/base.parent.cx,y=double(area.bottom)/base.parent.cy;
        auto r=base.rect;SetWindowPos(child,nullptr,int(r.left*x),int(r.top*y),int((r.right-r.left)*x),int((r.bottom-r.top)*y),SWP_NOZORDER|SWP_NOACTIVATE);
    }
    if(parent==mainWindow)for(int i=0;i<3;i++)SetWindowPos(GetDlgItem(parent,MinimizeId+i),HWND_TOP,area.right-(3-i)*captionButtonWidth,0,captionButtonWidth,titleHeight,SWP_NOACTIVATE);
}

static LRESULT CALLBACK dialogProc(HWND,UINT,WPARAM,LPARAM,UINT_PTR,DWORD_PTR);
static void layoutWelcome(HWND page){
    HWND bitmap=GetDlgItem(page,1200),title=GetDlgItem(page,1201),body=GetDlgItem(page,1202);
    if(!bitmap||!title||!body||GetPropW(page,L"TP_WELCOME"))return;
    SetPropW(page,L"TP_WELCOME",(HANDLE)1);
    RECT area{};GetClientRect(page,&area);const int portrait=area.right*32/100,margin=28;
    SetWindowPos(bitmap,nullptr,0,0,portrait,area.bottom,SWP_NOZORDER|SWP_NOACTIVATE);
    SetWindowPos(title,nullptr,portrait+margin,32,area.right-portrait-2*margin,74,SWP_NOZORDER|SWP_NOACTIVATE);
    SendMessageW(title,WM_SETFONT,(WPARAM)headingFont,TRUE);
    const int textWidth=area.right-portrait-2*margin;
    HDC dc=GetDC(body);auto old=SelectObject(dc,uiFont);RECT text{0,0,textWidth,0};auto content=textOf(body);DrawTextW(dc,content.c_str(),-1,&text,DT_CALCRECT|DT_WORDBREAK);SelectObject(dc,old);ReleaseDC(body,dc);
    SetWindowPos(body,nullptr,portrait+margin,119,textWidth,text.bottom+12,SWP_NOZORDER|SWP_NOACTIVATE);
    int next=119+text.bottom+32;
    for(HWND child=GetWindow(page,GW_CHILD);child;child=GetWindow(child,GW_HWNDNEXT)){
        if(className(child)!=L"Button")continue;
        RECT r{};GetClientRect(child,&r);SetWindowPos(child,nullptr,portrait+margin,next,textWidth,(r.bottom>24?r.bottom:28),SWP_NOZORDER|SWP_NOACTIVATE);next+=38;
    }
    for(HWND child=GetWindow(page,GW_CHILD);child;child=GetWindow(child,GW_HWNDNEXT)){
        RECT r{};GetWindowRect(child,&r);MapWindowPoints(nullptr,page,(POINT*)&r,2);layouts[child]=Layout{r,{area.right,area.bottom}};
    }
}
static void styleTree(HWND parent){
    for(HWND child=GetWindow(parent,GW_CHILD);child;child=GetWindow(child,GW_HWNDNEXT)){
        if(!GetPropW(child,L"TP_STYLED")){
            SetPropW(child,L"TP_STYLED",(HANDLE)1);
            auto name=className(child);
            LONG_PTR ex=GetWindowLongPtrW(child,GWL_EXSTYLE);SetWindowLongPtrW(child,GWL_EXSTYLE,ex&~WS_EX_CLIENTEDGE);
            SetWindowLongPtrW(child,GWL_STYLE,GetWindowLongPtrW(child,GWL_STYLE)&~WS_BORDER);
            SetWindowPos(child,nullptr,0,0,0,0,SWP_NOMOVE|SWP_NOSIZE|SWP_NOZORDER|SWP_NOACTIVATE|SWP_FRAMECHANGED);
            SetWindowTheme(child,L"",L"");
            if(name==L"#32770")SetWindowSubclass(child,dialogProc,1,0);else SetWindowSubclass(child,controlProc,1,0);
            if(name==L"Button"||name==L"Static"||name==L"Edit"||name==L"ListBox"||name.find(L"RichEdit")!=std::wstring::npos||name.find(L"RICHEDIT")!=std::wstring::npos)SendMessageW(child,WM_SETFONT,(WPARAM)uiFont,TRUE);
            if(name.find(L"RichEdit")!=std::wstring::npos||name.find(L"RICHEDIT")!=std::wstring::npos){SendMessageW(child,EM_SETBKGNDCOLOR,0,Surface);CHARFORMAT2W format{};format.cbSize=sizeof(format);format.dwMask=CFM_COLOR;format.crTextColor=Text;SendMessageW(child,EM_SETCHARFORMAT,SCF_ALL,(LPARAM)&format);}
            if(name==PROGRESS_CLASSW){SendMessageW(child,PBM_SETBKCOLOR,0,Surface);SendMessageW(child,PBM_SETBARCOLOR,0,Accent);}
            if(name==L"Static"&&(GetWindowLongPtrW(child,GWL_STYLE)&SS_TYPEMASK)==SS_BITMAP)SetWindowLongPtrW(child,GWL_STYLE,GetWindowLongPtrW(child,GWL_STYLE)|SS_REALSIZECONTROL);
            if(name==L"Static"&&((GetWindowLongPtrW(child,GWL_STYLE)&SS_TYPEMASK)==SS_ETCHEDHORZ))ShowWindow(child,SW_HIDE);
            RECT rect{},parentRect{};GetWindowRect(child,&rect);MapWindowPoints(nullptr,parent,(POINT*)&rect,2);GetClientRect(parent,&parentRect);
            if(name==L"Static"&&rect.bottom-rect.top<=3&&rect.right-rect.left>100)ShowWindow(child,SW_HIDE);
            layouts.emplace(child,Layout{rect,{parentRect.right,parentRect.bottom}});
            InvalidateRect(child,nullptr,TRUE);
        }
        styleTree(child);
    }
    if(parent==mainWindow){HWND divider=GetDlgItem(parent,1045);if(divider)ShowWindow(divider,SW_HIDE);}
    if(parent!=mainWindow)layoutWelcome(parent);
}

static LRESULT CALLBACK dialogProc(HWND window,UINT message,WPARAM wParam,LPARAM lParam,UINT_PTR,DWORD_PTR){
    if(message==WM_CTLCOLORDLG)return (LRESULT)backgroundBrush;
    if(message==WM_CTLCOLORSTATIC||message==WM_CTLCOLORBTN||message==WM_CTLCOLOREDIT||message==WM_CTLCOLORLISTBOX){HDC dc=(HDC)wParam;SetTextColor(dc,Text);SetBkColor(dc,message==WM_CTLCOLOREDIT||message==WM_CTLCOLORLISTBOX?Surface:Background);return (LRESULT)(message==WM_CTLCOLOREDIT||message==WM_CTLCOLORLISTBOX?surfaceBrush:backgroundBrush);}
    if(message==WM_ERASEBKGND){RECT rect{};GetClientRect(window,&rect);FillRect((HDC)wParam,&rect,backgroundBrush);return 1;}
    if(message==WM_PARENTNOTIFY||message==WM_SHOWWINDOW)PostMessageW(mainWindow,WM_APP+31,0,0);
    if(message==WM_APP+31){styleTree(mainWindow);return 0;}
    if(message==WM_SIZE){LRESULT result=DefSubclassProc(window,message,wParam,lParam);resizeChildren(window);return result;}
    if(window==mainWindow){
        if(message==WM_TIMER&&wParam==0x7f10){styleTree(window);return 0;}
        if(message==WM_NCHITTEST){POINT p{static_cast<short>(LOWORD(lParam)),static_cast<short>(HIWORD(lParam))};ScreenToClient(window,&p);RECT rect{};GetClientRect(window,&rect);if(p.y>=0&&p.y<titleHeight&&p.x<rect.right-3*captionButtonWidth)return HTCAPTION;}
        if(message==WM_COMMAND){const int id=LOWORD(wParam);if(id==MinimizeId){ShowWindow(window,SW_MINIMIZE);return 0;}if(id==MaximizeId){ShowWindow(window,IsZoomed(window)?SW_RESTORE:SW_MAXIMIZE);return 0;}if(id==CloseId){SendMessageW(window,WM_CLOSE,0,0);return 0;}}
        if(message==WM_GETMINMAXINFO){auto info=(MINMAXINFO*)lParam;MONITORINFO monitor{sizeof(monitor)};GetMonitorInfoW(MonitorFromWindow(window,MONITOR_DEFAULTTONEAREST),&monitor);info->ptMaxPosition={monitor.rcWork.left-monitor.rcMonitor.left,monitor.rcWork.top-monitor.rcMonitor.top};info->ptMaxSize={monitor.rcWork.right-monitor.rcWork.left,monitor.rcWork.bottom-monitor.rcWork.top};info->ptMinTrackSize={700,550};return 0;}
        if(message==WM_PAINT){PAINTSTRUCT paint{};HDC dc=BeginPaint(window,&paint);RECT area{};GetClientRect(window,&area);RECT bar{0,0,area.right,titleHeight};fill(dc,bar,Background);SetBkMode(dc,TRANSPARENT);SetTextColor(dc,Muted);auto font=SelectObject(dc,uiFont);RECT label{17,0,area.right-150,titleHeight};DrawTextW(dc,L"TP LAUNCHER",-1,&label,DT_VCENTER|DT_SINGLELINE);SelectObject(dc,font);line(dc,0,titleHeight-1,area.right,titleHeight-1,RGB(63,36,83));EndPaint(window,&paint);return 0;}
    }
    if(message==WM_NCDESTROY){layouts.erase(window);RemoveWindowSubclass(window,dialogProc,1);}
    return DefSubclassProc(window,message,wParam,lParam);
}

extern "C" __declspec(dllexport) void __cdecl Apply(HWND parent,int,wchar_t*,void*,void*){
    if(mainWindow)return;
    HMODULE pinned{};GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS|GET_MODULE_HANDLE_EX_FLAG_PIN,reinterpret_cast<LPCWSTR>(&Apply),&pinned);
    mainWindow=parent;
    HDC dc=GetDC(parent);const int dpi=GetDeviceCaps(dc,LOGPIXELSY);ReleaseDC(parent,dc);
    titleHeight=MulDiv(34,dpi,96);captionButtonWidth=MulDiv(45,dpi,96);uiFont=CreateFontW(-MulDiv(11,dpi,72),0,0,0,FW_NORMAL,FALSE,FALSE,FALSE,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,CLEARTYPE_QUALITY,DEFAULT_PITCH,L"Segoe UI");
    headingFont=CreateFontW(-MulDiv(17,dpi,72),0,0,0,FW_BOLD,FALSE,FALSE,FALSE,DEFAULT_CHARSET,OUT_DEFAULT_PRECIS,CLIP_DEFAULT_PRECIS,CLEARTYPE_QUALITY,DEFAULT_PITCH,L"Segoe UI");
    SetWindowSubclass(parent,dialogProc,1,0);
    SetClassLongPtrW(parent,GCLP_HBRBACKGROUND,(LONG_PTR)backgroundBrush);
    LONG_PTR style=GetWindowLongPtrW(parent,GWL_STYLE);SetWindowLongPtrW(parent,GWL_STYLE,(style&~(WS_CAPTION|WS_THICKFRAME))|WS_MINIMIZEBOX|WS_MAXIMIZEBOX|WS_SYSMENU);
    BOOL dark=TRUE;DwmSetWindowAttribute(parent,20,&dark,sizeof(dark));COLORREF border=RGB(65,36,87);DwmSetWindowAttribute(parent,34,&Background,sizeof(Background));DwmSetWindowAttribute(parent,35,&border,sizeof(border));
    SetWindowPos(parent,nullptr,0,0,0,0,SWP_NOMOVE|SWP_NOSIZE|SWP_NOZORDER|SWP_FRAMECHANGED);
    for(int i=0;i<3;i++)CreateWindowExW(0,L"Button",i==0?L"Minimize":i==1?L"Maximize":L"Close",WS_CHILD|WS_VISIBLE|WS_TABSTOP|BS_PUSHBUTTON,0,0,45,titleHeight,parent,(HMENU)(INT_PTR)(MinimizeId+i),pinned,nullptr);
    styleTree(parent);resizeChildren(parent);
    MONITORINFO monitor{sizeof(monitor)};RECT bounds{};
    if(GetMonitorInfoW(MonitorFromWindow(parent,MONITOR_DEFAULTTONEAREST),&monitor)&&GetWindowRect(parent,&bounds)){
        const int availableWidth=monitor.rcWork.right-monitor.rcWork.left,availableHeight=monitor.rcWork.bottom-monitor.rcWork.top;
        const int width=(bounds.right-bounds.left<availableWidth-24?bounds.right-bounds.left:availableWidth-24),height=(bounds.bottom-bounds.top<availableHeight-24?bounds.bottom-bounds.top:availableHeight-24);
        SetWindowPos(parent,nullptr,monitor.rcWork.left+(availableWidth-width)/2,monitor.rcWork.top+(availableHeight-height)/2,width,height,SWP_NOZORDER|SWP_NOACTIVATE);
    }
    SetTimer(parent,0x7f10,150,nullptr);InvalidateRect(parent,nullptr,TRUE);
}
