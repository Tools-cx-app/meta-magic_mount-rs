from telethon import TelegramClient
import asyncio
from telethon.sessions import StringSession
import os
import glob

API_ID = 611335
API_HASH = "d524b414d21f4d37f08684c1df41ac9c"

BOT_TOKEN = os.environ.get("BOT_TOKEN")
CHAT_ID = int(os.environ.get("CHAT_ID"))
COMMIT_URL = os.environ.get("COMMIT_URL")
RUN_ID = int(os.environ.get("RUN_ID"))
COMMIT_MESSAGE = os.environ.get("COMMIT_MESSAGE")
BOT_CI_SESSION = os.environ.get("BOT_CI_SESSION")
MSG_TEMPLATE = """
New push to Github
```
{commit_message}
```
See commit detail [here]({commit_url})
#ci_{run_id}
""".strip()

def get_caption():
    msg = MSG_TEMPLATE.format(
        commit_message=COMMIT_MESSAGE,
        commit_url=COMMIT_URL,
        run_id=RUN_ID,
    )
    if len(msg) > 1024:
        return COMMIT_URL
    return msg

def get_zip_files():
    return glob.glob('./output/*.zip')

async def send_telegram_message():
    zip_files = get_zip_files()
    
    if not zip_files:
        print("[-] Not found any zip files")
        return
        
    async with TelegramClient(StringSession(BOT_CI_SESSION), api_id=API_ID, api_hash=API_HASH) as client:
        await client.start(bot_token=BOT_TOKEN)
        print("[+] Caption: ")
        print(get_caption())
        print("---")
        print("[+] Sending")
        await client.send_file(
            entity=CHAT_ID,
            caption=get_caption(),
            file=zip_files,
            parse_mode="markdown",
        )

if __name__ == '__main__':
    asyncio.run(send_telegram_message())
