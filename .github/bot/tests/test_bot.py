# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from bot.config import PARSING_MAX_LEN
from bot.parsing import parse_git_log
from bot.gh_helpers import (
    load_event_payload,
    get_commits_from_event,
    get_url_from_event,
    get_git_root,
    get_git_log,
    run_git,
)
from bot.msg_gen import generate_msg_ci, generate_msg_release


class TestParsing(unittest.TestCase):
    def test_parse_git_log_empty(self):
        self.assertEqual(parse_git_log(""), "No commit found")
        self.assertEqual(parse_git_log("   \n\n  "), "No commit found")

    def test_parse_git_log_escape_html(self):
        log = "1234567 feat: add <script> & 'styles'"
        res = parse_git_log(log)
        self.assertIn("&lt;script&gt;", res)
        self.assertIn("&amp;", res)
        self.assertNotIn("<script>", res)

    def test_parse_git_log_reversal_and_truncation(self):
        # Input is newest first.
        lines = [f"{i:07d} commit {i}" for i in range(100)]
        log = "\n".join(lines)
        res = parse_git_log(log)
        self.assertTrue(res.startswith("..."))
        self.assertIn("more commits...", res)
        self.assertLessEqual(len(res), PARSING_MAX_LEN + 100)


class TestEventPayload(unittest.TestCase):
    def test_load_event_payload(self):
        with tempfile.NamedTemporaryFile(
            "w", delete=False, suffix=".json", encoding="utf-8"
        ) as f:
            json.dump({"ref": "refs/heads/master", "before": "abc", "commits": []}, f)
            temp_path = f.name

        try:
            with patch("bot.gh_helpers.settings") as mock_settings:
                mock_settings.github_event_path = temp_path
                payload = load_event_payload()
                self.assertEqual(payload.get("ref"), "refs/heads/master")
                self.assertEqual(payload.get("before"), "abc")
        finally:
            os.remove(temp_path)

    def test_get_commits_from_event_normal(self):
        payload = {
            "commits": [
                {"id": "11111112222222", "message": "first commit\n\ndetails"},
                {"id": "33333334444444", "message": "second commit"},
            ]
        }
        commits = get_commits_from_event(payload)
        # Newest first.
        self.assertEqual(commits[0], "3333333 second commit")
        self.assertEqual(commits[1], "1111111 first commit")

    def test_get_commits_from_event_head_commit_fallback(self):
        payload = {
            "commits": [],
            "head_commit": {"id": "abcdef123456", "message": "single head commit\nline2"},
        }
        commits = get_commits_from_event(payload)
        self.assertEqual(commits, ["abcdef1 single head commit"])

    def test_get_url_from_event_compare(self):
        payload = {
            "before": "1" * 40,
            "compare": "https://github.com/foo/bar/compare/111...222",
        }
        self.assertEqual(
            get_url_from_event(payload),
            "https://github.com/foo/bar/compare/111...222",
        )

    def test_get_url_from_event_initial_push_zeros(self):
        payload = {
            "before": "0" * 40,
            "compare": "https://github.com/foo/bar/compare/000...222",
            "head_commit": {"url": "https://github.com/foo/bar/commit/222"},
        }
        self.assertEqual(get_url_from_event(payload), "https://github.com/foo/bar/commit/222")


class TestGitHelpers(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        repo = Path(self.temp_dir.name)
        self.repo = repo
        env = dict(os.environ)
        env["GIT_AUTHOR_NAME"] = "Test Bot"
        env["GIT_AUTHOR_EMAIL"] = "bot@example.com"
        env["GIT_COMMITTER_NAME"] = "Test Bot"
        env["GIT_COMMITTER_EMAIL"] = "bot@example.com"
        self.git_env = env
        self._git("init", "--initial-branch=main")
        (repo / "file.txt").write_text("one\n", encoding="utf-8")
        self._git("add", "file.txt")
        self._git("commit", "-m", "base commit")
        (repo / "file.txt").write_text("two\n", encoding="utf-8")
        self._git("commit", "-am", "head commit")
        self.head_sha = self._git("rev-parse", "HEAD").strip()
        self.base_sha = self._git("rev-parse", "HEAD^").strip()

    def tearDown(self):
        self.temp_dir.cleanup()

    def _git(self, *args: str) -> str:
        return subprocess.check_output(
            ["git", *args],
            cwd=self.temp_dir.name,
            env=self.git_env,
            text=True,
        )

    async def test_run_git_returns_output(self):
        with patch("bot.gh_helpers.get_git_root", AsyncMock(return_value=self.repo)):
            self.assertEqual(
                await run_git("rev-parse", "--short", "HEAD"),
                f"{self.head_sha[:7]}\n",
            )

    async def test_get_git_log(self):
        with patch("bot.gh_helpers.get_git_root", AsyncMock(return_value=self.repo)):
            self.assertEqual(
                await get_git_log(self.base_sha, self.head_sha),
                f"{self.head_sha[:7]} head commit",
            )
            self.assertEqual(await get_git_log(self.head_sha, self.head_sha), "")

    async def test_get_git_log_invalid_range(self):
        with patch("bot.gh_helpers.get_git_root", AsyncMock(return_value=self.repo)):
            self.assertEqual(await get_git_log("missing-base", self.head_sha), "")

    async def test_get_git_root_uses_workspace(self):
        with patch("bot.gh_helpers.cache") as mock_cache, \
             patch.dict(os.environ, {"GITHUB_WORKSPACE": str(self.repo)}):
            mock_cache.git_root = None
            self.assertEqual(await get_git_root(), self.repo)


class TestMsgGen(unittest.IsolatedAsyncioTestCase):
    def _settings(self, mock_settings, **overrides):
        values = {
            "github_sha": "head_sha",
            "github_ref_name": "master",
            "github_ref": "refs/heads/master",
            "github_repository": "test/repo",
            "run_no": 42,
            "run_id": 999,
        }
        values.update(overrides)
        for name, value in values.items():
            setattr(mock_settings, name, value)

    async def test_generate_msg_ci_payload_before(self):
        payload = {
            "before": "base_sha",
            "compare": "https://github.com/test/repo/compare/base_sha...head_sha",
        }
        with patch("bot.msg_gen.load_event_payload", return_value=payload), \
             patch("bot.msg_gen.get_git_log", AsyncMock(return_value="base_sha commit A")), \
             patch("bot.msg_gen.settings") as mock_settings:
            self._settings(mock_settings)
            msg = await generate_msg_ci()
            self.assertIn("commit A", msg)
            self.assertIn("https://github.com/test/repo/compare/base_sha...head_sha", msg)
            self.assertIn("#ci_42", msg)

    async def test_generate_msg_ci_event_commits(self):
        payload = {
            "commits": [{"id": "1234567890", "message": "payload commit"}],
            "compare": "https://github.com/test/repo/compare/old...new",
            "before": "old",
        }
        with patch("bot.msg_gen.load_event_payload", return_value=payload), \
             patch("bot.msg_gen.get_git_log", AsyncMock(return_value="")), \
             patch("bot.msg_gen.settings") as mock_settings:
            self._settings(mock_settings, github_sha="new")
            msg = await generate_msg_ci()
            self.assertIn("payload commit", msg)
            self.assertIn("https://github.com/test/repo/compare/old...new", msg)

    async def test_generate_msg_ci_git_history(self):
        with patch("bot.msg_gen.load_event_payload", return_value={}), \
             patch("bot.msg_gen.get_git_log", AsyncMock(return_value="abc1234 git commit")), \
             patch("bot.msg_gen.settings") as mock_settings:
            self._settings(mock_settings)
            msg = await generate_msg_ci()
            self.assertIn("git commit", msg)
            self.assertIn("https://github.com/test/repo/commit/head_sha", msg)

    async def test_generate_msg_ci_single_commit_fallback(self):
        with patch("bot.msg_gen.load_event_payload", return_value={}), \
             patch("bot.msg_gen.get_git_log", AsyncMock(return_value="")), \
             patch(
                 "bot.msg_gen.run_git",
                 AsyncMock(return_value="single commit fallback\nbody"),
             ), \
             patch("bot.msg_gen.settings") as mock_settings:
            self._settings(mock_settings, github_sha="abcdef123456")
            msg = await generate_msg_ci()
            self.assertIn("single commit fallback", msg)
            self.assertIn("https://github.com/test/repo/commit/abcdef123456", msg)

    async def test_generate_msg_release_tag(self):
        payload = {"repository": {"full_name": "test/repo"}}
        with patch("bot.msg_gen.load_event_payload", return_value=payload), \
             patch("bot.msg_gen.settings") as mock_settings:
            self._settings(
                mock_settings,
                github_ref_name="v1.2.3",
                github_ref="refs/tags/v1.2.3",
            )
            msg = await generate_msg_release()
            self.assertIn("v1.2.3", msg)
            self.assertIn("https://github.com/test/repo/releases/tag/v1.2.3", msg)

    async def test_generate_msg_release_git_tag_fallback(self):
        with patch("bot.msg_gen.load_event_payload", return_value={}), \
             patch("bot.msg_gen.run_git", AsyncMock(return_value="v2.0.0\n")), \
             patch("bot.msg_gen.settings") as mock_settings:
            self._settings(
                mock_settings,
                github_ref_name="",
                github_ref="refs/tags/v2.0.0",
            )
            msg = await generate_msg_release()
            self.assertIn("v2.0.0", msg)
            self.assertIn("https://github.com/test/repo/releases/tag/v2.0.0", msg)


if __name__ == "__main__":
    unittest.main()
