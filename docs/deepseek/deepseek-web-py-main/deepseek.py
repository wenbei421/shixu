import os
import webbrowser
from optimisewait import optimiseWait, set_autopath, set_altpath
import win32clipboard
from time import sleep
import pyautogui
import pywintypes

set_autopath(os.path.dirname(__file__))

def set_clipboard(text, retries=3, delay=0.2):
    for i in range(retries):
        try:
            win32clipboard.OpenClipboard()
            win32clipboard.EmptyClipboard()
            try:
                win32clipboard.SetClipboardText(str(text))
            except Exception:
                # Fallback for Unicode characters
                win32clipboard.SetClipboardData(win32clipboard.CF_UNICODETEXT, str(text).encode('utf-16le'))
            win32clipboard.CloseClipboard()
            return  # Success
        except pywintypes.error as e:
            if e.winerror == 5:  # Access is denied
                print(f"Clipboard access denied. Retrying... (Attempt {i+1}/{retries})")
                sleep(delay)
            else:
                raise  # Re-raise other pywintypes errors
        except Exception as e:
            raise  # Re-raise other exceptions
    print(f"Failed to set clipboard after {retries} attempts.")


def deepseek(prompt):
    url = 'https://chat.deepseek.com/'
    
    webbrowser.open_new_tab(url)

    optimiseWait('dsmessage')

    # Extract and handle base64 images before logging, will get working in feature update
    """ if 'messages' in request_json:
        for message in request_json['messages']:
            content = message.get('content', [])
            if isinstance(content, list):
                for item in content:
                    if isinstance(item, dict) and item.get('type') == 'image_url':
                        image_url = item.get('image_url', {}).get('url', '')
                        if image_url.startswith('data:image'):
                            set_clipboard_image(image_url)
                            pyautogui.hotkey('ctrl','v')
                            # Remove image data from logs
                            item['image_url']['url'] = '[IMAGE DATA REMOVED]'
                            sleep(7) """
    

    # Send instructions to Claude

    set_clipboard(prompt)   
    pyautogui.hotkey('ctrl','v')

    sleep(1)

    optimiseWait('dsrun')

    optimiseWait('dscopy')    
    
    pyautogui.hotkey('ctrl','w')
    
    pyautogui.hotkey('alt','tab')

    # Get Claude's response
    win32clipboard.OpenClipboard()
    response = win32clipboard.GetClipboardData()
    win32clipboard.CloseClipboard()

    return response