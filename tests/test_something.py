import subprocess
import base64
import zipfile
from pathlib import Path
from subprocess import Popen

import pytest


def bot_binary_path():
    for path in [
        Path.cwd() / "target" / "debug" / "xdcstore",
        Path.cwd() / "xdcstore" / "xdcstore",
    ]:
        if path.exists():
            return path
    pytest.fail("could not determine bot_binary_path")


def bot_assets_path():
    for path in [
        Path.cwd() / "assets",
        Path.cwd() / "xdcstore" / "assets",
    ]:
        if path.exists():
            return path
    pytest.fail("could not determine bot_assets_path")


@pytest.fixture
def bot_home_path(tmp_path_factory):
    """HOME path for the bot."""
    return tmp_path_factory.mktemp("bothome")


class BotProcess:
    addr: str
    process: Popen

    def __init__(self, addr, password, binary_path, home_path):
        self.addr = addr
        self.password = password
        self.binary_path = binary_path
        self.home_path = home_path

    def start(self, **kwargs):
        self.process = Popen(
            [self.binary_path, "start"],
            cwd=self.binary_path.parent,
            env={
                "HOME": str(self.home_path),
                "addr": self.addr,
                "mail_pw": self.password,
                "RUST_LOG": "xdcstore=info",
                **kwargs,
            },
        )

    def stop(self):
        self.process.terminate()

    def install_examples(self):
        subprocess.run(
            [
                self.binary_path,
                "import",
                Path.cwd() / "example-xdcs",
            ],
            cwd=self.binary_path.parent,
            env={
                "RUST_LOG": "xdcstore=info",
                "HOME": str(self.home_path),
                "addr": self.addr,
                "mail_pw": self.password,
            },
            check=True,
        )

    def __del__(self):
        self.stop()


@pytest.fixture
def storebot_stopped(acfactory, bot_home_path):
    """Stopped store bot without any apps."""
    config = acfactory.get_next_liveconfig()

    return BotProcess(
        config["addr"], config["mail_pw"], bot_binary_path(), bot_home_path
    )


@pytest.fixture
def storebot(acfactory, storebot_stopped):
    """Store bot without any apps."""
    storebot_stopped.start()
    return storebot_stopped


@pytest.fixture
def storebot_example(acfactory, storebot_stopped):
    """Store bot with imported example apps."""
    storebot_stopped.install_examples()
    storebot_stopped.start()
    return storebot_stopped


def test_welcome_message(acfactory, storebot):
    """Test that the bot responds with a welcome message."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    assert "Welcome to the webxdc store!" in msg_in.text


def test_update(acfactory, storebot):
    """Test that the bot sends initial update and responds to update requests."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    assert "Welcome" in msg_in.text

    assert msg_in.is_webxdc()
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 1
    assert status_updates[0]["payload"] == {
        "type": "Update",
        "app_infos": [],
        "serial": 0,
        "old_serial": 0,
        "updating": [],
    }

    # Request updates.
    assert msg_in.send_status_update(
        {"payload": {"type": "UpdateRequest", "serial": 0}}, "update"
    )
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")

    # Receive a response.
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 3
    payload = status_updates[-1]["payload"]
    assert payload == {"type": "Update", "app_infos": [], "serial": 0, "old_serial": 0, "updating": []}


def test_update_advanced(acfactory, storebot_example):
    """Test that the bot sends initial update and responds to update requests."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot_example.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")

    # Request updates.
    # calendar is outdated, hextris not
    assert msg_in.send_status_update(
        {
            "payload": {
                "type": "UpdateRequest",
                "serial": 0,
                "apps": [("webxdc-calendar", "v1.0.1"), ("webxdc-hextris", "v1.0.2")],
            }
        },
        "update",
    )
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")

    # Receive a response.
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 3
    payload = status_updates[-1]["payload"]
    assert payload["updating"] == ["webxdc-calendar"]

    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 4
    payload = status_updates[-1]["payload"]
    assert payload["app_id"] == "webxdc-calendar"
    assert payload["type"] == "DownloadOkay"


def test_partial_update(acfactory, storebot_example):
    """Test that the bot sends initial update and responds to update requests."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot_example.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    serial = msg_in.get_status_updates()[-1]["payload"]["serial"]

    sources_path = Path.cwd() / "example-xdcs" / "xdcget.lock"
    new_lines = []
    old_lines = []
    with open(sources_path, "r+") as f:
        old_lines = f.readlines()
        # Replace description and change the tag
        new_lines = [
            'description = "pupu"\n'
            if line.startswith("description")
            else 'tag_name = "v100"\n'
            if line.startswith("tag_name")
            else line
            for line in old_lines
        ]

    with open(sources_path, "w") as f:
        f.writelines(new_lines)

    storebot_example.install_examples()

    # restore old lock file
    with open(sources_path, "w") as f:
        f.writelines(old_lines)

    # Request updates.
    # dc-calendar is outdated by 1 version, dc-hextris not
    assert msg_in.send_status_update(
        {
            "payload": {
                "type": "UpdateRequest",
                "serial": serial,
                "apps": [],
            }
        },
        "update",
    )
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")

    # Receive a response.
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 3
    payload = status_updates[-1]["payload"]
    assert payload["app_infos"][0] == {
        "app_id": "webxdc-2048",
        "description": "pupu",
        "tag_name": "v100",
    }


def test_import(acfactory, storebot_example):
    """Test that import works."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot_example.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")

    assert msg_in.is_webxdc()
    status_updates = msg_in.get_status_updates()
    assert len(status_updates) == 1
    payload = status_updates[0]["payload"]
    app_infos = payload["app_infos"]
    assert len(app_infos) == 4


def test_version(acfactory, storebot):
    """Test /version command."""

    (ac1,) = acfactory.get_online_accounts(1)

    version_text = subprocess.run(
        [bot_binary_path(), "version"], capture_output=True, check=True
    ).stdout.decode()

    bot_contact = ac1.create_contact(storebot.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("/version")

    msg_in = ac1.wait_next_incoming_message()

    assert msg_in.text + "\n" == version_text


def test_download(acfactory, storebot_example):
    """Test that download works."""
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot_example.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")

    status_updates = msg_in.get_status_updates()
    payload = status_updates[0]["payload"]
    app_infos = payload["app_infos"]
    xdc_2040 = [xdc for xdc in app_infos if xdc["name"] == "2048"][0]

    assert msg_in.send_status_update(
        {"payload": {"type": "Download", "app_id": xdc_2040["app_id"]}}, ""
    )

    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")

    # Test download response for existing app.
    status_updates = msg_in.get_status_updates()
    payload = status_updates[2]["payload"]
    assert payload["type"] == "DownloadOkay"
    assert payload["app_id"] == xdc_2040["app_id"]
    assert payload["name"] == "2048"
    with open(str(Path.cwd()) + "/example-xdcs/webxdc-2048-v1.2.1.xdc", "rb") as f:
        assert payload["data"] == base64.b64encode(f.read()).decode("ascii")

    # Test download response for non-existing app.
    assert msg_in.send_status_update(
        {"payload": {"type": "Download", "app_id": "xxx"}}, "update"
    )
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")
    status_updates = msg_in.get_status_updates()
    payload = status_updates[4]["payload"]
    assert payload["type"] == "DownloadError"
    assert payload["app_id"] == "xxx"


def update_manifest_tag_name(bot_path, new_tag_name):
    temp_zip_file = bot_path / "temp.xdc"
    zip_file_path = bot_path / "store.xdc"
    with zipfile.ZipFile(zip_file_path, "r") as zip_read, zipfile.ZipFile(
        temp_zip_file, "w"
    ) as zip_write:
        for file in zip_read.infolist():
            if file.filename != "manifest.toml":
                zip_write.writestr(file, zip_read.read(file.filename))

        manifest_filename = "manifest.toml"
        manifest_content = zip_read.read(manifest_filename).decode("utf-8")

        updated_content = ""
        for line in manifest_content.split("\n"):
            if line.startswith("tag_name ="):
                updated_content += f"tag_name = {new_tag_name}\n"
            else:
                updated_content += line + "\n"

        updated_manifest = zipfile.ZipInfo(manifest_filename)
        updated_manifest.compress_type = zipfile.ZIP_DEFLATED
        zip_write.writestr(updated_manifest, updated_content.encode("utf-8"))

    temp_zip_file.replace(zip_file_path)


def test_frontend_update(acfactory, storebot):
    (ac1,) = acfactory.get_online_accounts(1)

    bot_contact = ac1.create_contact(storebot.addr)
    bot_chat = bot_contact.create_chat()
    bot_chat.send_text("hi!")

    msg_in = ac1.wait_next_incoming_message()
    ac1._evtracker.get_matching(
        "DC_EVENT_WEBXDC_STATUS_UPDATE"
    )  # Inital store hydration
    assert msg_in.is_webxdc()

    update_manifest_tag_name(storebot.home_path / ".config" / "xdcstore", '"v1000"')

    # Start the bot again to load the newer store.xdc version
    storebot.stop()
    storebot.start(XDCSTORE_KEEP_ASSETS="yes")

    msg_in.send_status_update({"payload": {"type": "UpdateRequest", "serial": 0}}, "")
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")  # Self-sent
    ac1._evtracker.get_matching(
        "DC_EVENT_WEBXDC_STATUS_UPDATE"
    )  # Update needed response

    # Test that the bot sends an outdated response
    status_updates = msg_in.get_status_updates()
    payload = status_updates[2]["payload"]
    assert payload == {"type": "Outdated", "critical": True, "tag_name": "v1000"}

    # In store.xdc the update button should send this message
    msg_in.send_status_update({"payload": {"type": "UpdateWebxdc"}}, "")

    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")  # Self-sent
    ac1._evtracker.get_matching("DC_EVENT_WEBXDC_STATUS_UPDATE")  # Update sent response

    # Test that the bot sends an a download confirmation
    status_updates = msg_in.get_status_updates()
    payload = status_updates[4]["payload"]
    assert payload == {"type": "UpdateSent"}

    # Test that the bots sends a new version of the store
    msg_in = ac1.wait_next_incoming_message()
    assert msg_in.is_webxdc()
