from telethon import TelegramClient
import asyncio
from telethon.sessions import StringSession
import os
import glob
import requests

API_ID = 611335
API_HASH = "d524b414d21f4d37f08684c1df41ac9c"

# 环境变量
BOT_TOKEN = os.environ.get("BOT_TOKEN")
CHAT_ID = os.environ.get("CHAT_ID")
try:
    CHAT_ID = int(CHAT_ID)
except (ValueError, TypeError):
    pass
COMMIT_URL = os.environ.get("COMMIT_URL")
RUN_ID = int(os.environ.get("RUN_ID"))
COMMIT_MESSAGE = os.environ.get("COMMIT_MESSAGE")
BOT_CI_SESSION = os.environ.get("BOT_CI_SESSION")
GITHUB_REPOSITORY = os.environ.get("GITHUB_REPOSITORY")
GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN")
CURRENT_COMMIT = os.environ.get("GITHUB_SHA")

MSG_TEMPLATE = """
New push to Github
```
{commit_message}
```
See commit detail [here]({commit_url})
#ci_{run_id}
""".strip()


def get_last_successful_ci_commit():
    """获取最近一次成功的ci-build workflow的commit hash"""
    if not GITHUB_REPOSITORY or not GITHUB_TOKEN:
        print("[-] Missing required GitHub environment variables")
        return None

    url = f"https://api.github.com/repos/{GITHUB_REPOSITORY}/actions/runs"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github.v3+json",
    }
    params = {
        "event": "push",
        "status": "success",
        "per_page": 1,
        "sort": "created",
        "direction": "desc",
        "workflow": "ci-build",
    }

    try:
        response = requests.get(url, headers=headers, params=params)
        response.raise_for_status()
        runs = response.json().get("workflow_runs", [])
        if runs:
            print(
                f"[+] Found last successful CI run: {runs[0]['id']} - {runs[0]['head_sha']}"
            )
            return runs[0]["head_sha"]
        print("[-] No successful CI runs found")
        return None
    except Exception as e:
        print(f"[-] Error getting last successful CI: {e}")
        return None


def get_commit_messages(from_commit, to_commit):
    """获取两个commit之间的所有commit messages"""
    if not GITHUB_REPOSITORY or not GITHUB_TOKEN:
        print("[-] Missing required GitHub environment variables")
        return []

    url = f"https://api.github.com/repos/{GITHUB_REPOSITORY}/compare/{from_commit}...{to_commit}"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github.v3+json",
    }

    try:
        response = requests.get(url, headers=headers)
        response.raise_for_status()
        commits = response.json().get("commits", [])
        return [commit.get("commit", {}).get("message") for commit in commits]
    except Exception as e:
        print(f"[-] Error getting commit messages: {e}")
        return []


def get_caption():
    """生成消息标题"""
    last_successful_commit = get_last_successful_ci_commit()

    if last_successful_commit and CURRENT_COMMIT:
        commit_messages = get_commit_messages(last_successful_commit, CURRENT_COMMIT)
        if commit_messages:
            combined_message = "\n\n".join(commit_messages)
            compare_url = (
                f"https://github.com/{GITHUB_REPOSITORY}/compare/{last_successful_commit}...{CURRENT_COMMIT}"
                if GITHUB_REPOSITORY
                else COMMIT_URL
            )

            msg = MSG_TEMPLATE.format(
                commit_message=combined_message,
                commit_url=compare_url,
                run_id=RUN_ID,
            )

            if len(msg) > 1024:
                recent_messages = "\n\n".join(commit_messages[-5:])
                msg = MSG_TEMPLATE.format(
                    commit_message=recent_messages + "\n\n... (more commits)",
                    commit_url=compare_url,
                    run_id=RUN_ID,
                )
                if len(msg) > 1024:
                    return compare_url
            return msg

    # 回退方案
    msg = MSG_TEMPLATE.format(
        commit_message=COMMIT_MESSAGE,
        commit_url=COMMIT_URL,
        run_id=RUN_ID,
    )
    return COMMIT_URL if len(msg) > 1024 else msg


def get_zip_files():
    """获取zip文件列表"""
    return glob.glob("./output/*.zip")


async def send_telegram_message():
    """发送Telegram消息"""
    zip_files = get_zip_files()
    if not zip_files:
        print("[-] No zip files found")
        return

    # 创建客户端
    try:
        session = StringSession(BOT_CI_SESSION) if BOT_CI_SESSION else StringSession()
    except Exception:
        session = StringSession()

    # 创建客户端并直接使用bot_token启动
    client = TelegramClient(session, api_id=API_ID, api_hash=API_HASH)
    try:
        await client.start(bot_token=BOT_TOKEN)
        print("[+] Bot started successfully")

        # 获取实体
        try:
            entity = await client.get_entity(CHAT_ID)
            print(f"[+] Successfully got entity for CHAT_ID: {CHAT_ID}")
        except Exception as e:
            print(f"[-] Error getting entity: {e}")
            print("[+] Please ensure the bot has access to the chat")
            return

        # 发送消息
        caption = get_caption()
        print("[+] Caption: ")
        print(caption)
        print("---")
        print("[+] Sending")

        try:
            await client.send_file(
                entity=entity,
                caption=caption,
                file=zip_files,
                parse_mode="markdown",
            )
            print("[+] Message sent successfully")
        except Exception as e:
            print(f"[-] Failed to send message: {e}")
            return

        # 导出session
        if not BOT_CI_SESSION:
            session_string = client.session.save()
            with open("./session_string.txt", "w") as f:
                f.write(session_string)
            print("[+] Session string written to session_string.txt")
    finally:
        await client.disconnect()


if __name__ == "__main__":
    asyncio.run(send_telegram_message())
