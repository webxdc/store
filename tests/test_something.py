import shutil
import subprocess
from pathlib import Path
from subprocess import Popen

import pytest


class BotProcess:
    addr: str
    process: Popen

    def __init__(self, addr, password, path):
        self.addr = addr
        self.path = path

        self.process = Popen(
            [Path.cwd() / "target/debug/github-bot", "start"],
            cwd=path,
            env={"addr": addr, "mail_pw": password, "RUST_LOG": "github_bot=trace"},
        )

    def __del__(self):
        self.process.terminate()


@pytest.fixture
def bot_path(tmp_path):
    # Copy bot-data to the bot working directory.
    shutil.copytree("bot-data", tmp_path / "bot-data")

    return tmp_path


@pytest.fixture
def storebot(acfactory, bot_path):
    """Store bot without any apps."""
    config = acfactory.get_next_liveconfig()

    return BotProcess(config["addr"], config["mail_pw"], bot_path)


@pytest.fixture
def storebot_example(acfactory, bot_path):
    """Store bot with imported example apps."""
    config = acfactory.get_next_liveconfig()

    shutil.copytree("example-xdcs", bot_path / "import")
    (bot_path / "import" / "README.md").unlink()
    res = subprocess.run(
        [Path.cwd() / "target/debug/github-bot", "import"],
        cwd=bot_path,
        env={"RUST_LOG": "github_bot=trace"},
    )
    res.check_returncode()

    return BotProcess(config["addr"], config["mail_pw"], bot_path)


def test_welcome_message(acfactory, storebot):
    """Test that the bot responds with a welcome message."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    assert msg_in.text == "Welcome to the appstore bot!"


def test_update(acfactory, storebot):
    """Test that the bot sends initial update and responds to update requests."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    assert msg_in.text == "Welcome to the appstore bot!"

    assert msg_in.is_webxdc()
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 1
    assert status_updates[0]["payload"] == {"app_infos": [], "serial": 0}

    # Request updates.
    assert msg_in.send_status_update(
        {"payload": {"request_type": "Update", "data": 0}}, "update"
    )
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")

    # Receive a response.
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 3
    payload = status_updates[-1]["payload"]
    assert payload == {"app_infos": [], "serial": 0}


def test_import(acfactory, storebot_example):
    """Test that import works."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot_example.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    assert msg_in.text == "Welcome to the appstore bot!"

    assert msg_in.is_webxdc()
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 1
    payload = status_updates[0]["payload"]
    app_infos = payload["app_infos"]
    assert len(app_infos) == 4
