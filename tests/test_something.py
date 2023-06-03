from subprocess import Popen
from pathlib import Path
from dataclasses import dataclass
import shutil

import pytest

class BotProcess:
    addr: str
    process: Popen

    def __init__(self, addr, password, path):
        self.addr = addr
        self.path = path

        # Copy bot-data to the bot working directory.
        shutil.copytree("bot-data", path / "bot-data")
        self.process = Popen(
            [Path.cwd() / "target/debug/github-bot", "start"],
            cwd=path,
            env={"addr": addr, "mail_pw": password, "RUST_LOG": "github_bot=trace"},
        )

    def __del__(self):
        self.process.terminate()

@pytest.fixture
def storebot(acfactory, tmp_path):
    config = acfactory.get_next_liveconfig()

    return BotProcess(config["addr"], config["mail_pw"], tmp_path)


def test_welcome_message(acfactory, storebot):
    ac1, = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    assert msg_in.text == "Welcome to the appstore bot!"
