# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

import json
import os
import tempfile
import unittest
from unittest.mock import patch, AsyncMock

from bot.config import PARSING_MAX_LEN
from bot.parsing import parse_git_log
from bot.gh_helpers import (
    load_event_payload,
    get_commits_from_event,
    get_url_from_event,
    get_last_ci_run,
    get_git_log,
)
from bot.msg_gen import generate_msg_ci


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
        # input is newest first
        lines = [f"{i:07d} commit {i}" for i in range(100)]
        log = "\n".join(lines)
        res = parse_git_log(log)
        self.assertTrue(res.startswith("..."))
        self.assertIn("more commits...", res)
        self.assertLessEqual(len(res), PARSING_MAX_LEN + 100)


class TestEventPayload(unittest.TestCase):
    def test_load_event_payload(self):
        with tempfile.NamedTemporaryFile("w", delete=False, suffix=".json", encoding="utf-8") as f:
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
        # newest first
        self.assertEqual(len(commits), 2)
        self.assertEqual(commits[0], "3333333 second commit")
        self.assertEqual(commits[1], "1111111 first commit")

    def test_get_commits_from_event_head_commit_fallback(self):
        payload = {
            "commits": [],
            "head_commit": {"id": "abcdef123456", "message": "single head commit\nline2"},
        }
        commits = get_commits_from_event(payload)
        self.assertEqual(len(commits), 1)
        self.assertEqual(commits[0], "abcdef1 single head commit")

    def test_get_url_from_event_compare(self):
        payload = {
            "before": "1111111111111111111111111111111111111111",
            "compare": "https://github.com/foo/bar/compare/111...222",
        }
        self.assertEqual(get_url_from_event(payload), "https://github.com/foo/bar/compare/111...222")

    def test_get_url_from_event_initial_push_zeros(self):
        payload = {
            "before": "0000000000000000000000000000000000000000",
            "compare": "https://github.com/foo/bar/compare/000...222",
            "head_commit": {"url": "https://github.com/foo/bar/commit/222"},
        }
        self.assertEqual(get_url_from_event(payload), "https://github.com/foo/bar/commit/222")


class TestGhHelpers(unittest.IsolatedAsyncioTestCase):
    async def test_get_last_ci_run_skips_newer_and_same_commit(self):
        runs = [
            {"id": 205, "event": "push", "head_sha": "new_sha", "conclusion": "success"},
            {"id": 200, "event": "push", "head_sha": "current_sha", "conclusion": "success"},
            {"id": 199, "event": "push", "head_sha": "current_sha", "conclusion": "failure"},
            {"id": 198, "event": "push", "head_sha": "prev_sha", "conclusion": "success"},
        ]
        with patch("bot.gh_helpers.list_workflow_runs", AsyncMock(return_value={"workflow_runs": runs, "total_count": 4})):
            # Looking for run before id 200, and with commit different from current_sha
            res = await get_last_ci_run(before_run_id=200, before_commit="current_sha")
            self.assertIsNotNone(res)
            run, success = res
            self.assertEqual(run["id"], 198)
            self.assertTrue(success)

    async def test_get_last_ci_run_in_progress(self):
        runs = [
            {"id": 198, "event": "push", "head_sha": "prev_sha", "conclusion": None, "status": "in_progress"},
        ]
        with patch("bot.gh_helpers.list_workflow_runs", AsyncMock(return_value={"workflow_runs": runs, "total_count": 1})):
            res = await get_last_ci_run(before_run_id=200, before_commit="current_sha")
            self.assertIsNotNone(res)
            run, success = res
            self.assertEqual(run["id"], 198)
            self.assertFalse(success)

    async def test_get_git_log_safe(self):
        # Identical base and head
        res = await get_git_log("sha1", "sha1")
        self.assertEqual(res, "")

        # Normal comparison
        compare_data = {
            "total_commits": 1,
            "commits": [{"sha": "aabbccddee", "commit": {"message": "commit msg\n\nmore"}}],
        }
        with patch("bot.gh_helpers.compare_commit", AsyncMock(return_value=compare_data)):
            res = await get_git_log("sha1", "sha2")
            self.assertEqual(res, "aabbccd commit msg")

        # API error handling
        with patch("bot.gh_helpers.compare_commit", AsyncMock(side_effect=RuntimeError("API error"))):
            res = await get_git_log("sha1", "sha2")
            self.assertEqual(res, "")


class TestMsgGen(unittest.IsolatedAsyncioTestCase):
    async def test_generate_msg_ci_primary_success(self):
        with patch("bot.msg_gen.get_last_success_commit", AsyncMock(return_value="base_sha")), \
             patch("bot.msg_gen.get_git_log", AsyncMock(return_value="base_sha commit A")), \
             patch("bot.msg_gen.settings") as mock_settings:
            mock_settings.github_sha = "head_sha"
            mock_settings.github_ref_name = "master"
            mock_settings.github_repository = "test/repo"
            mock_settings.run_no = 42
            mock_settings.run_id = 999
            msg = await generate_msg_ci()
            self.assertIn("commit A", msg)
            self.assertIn("https://github.com/test/repo/compare/base_sha...head_sha", msg)
            self.assertIn("#ci_42", msg)

    async def test_generate_msg_ci_event_payload_backup(self):
        # Primary strategy fails (returns None)
        with patch("bot.msg_gen.get_last_success_commit", AsyncMock(return_value=None)), \
             patch("bot.msg_gen.load_event_payload", return_value={
                 "commits": [{"id": "1234567890", "message": "payload commit"}],
                 "compare": "https://github.com/test/repo/compare/old...new",
                 "before": "old",
             }), \
             patch("bot.msg_gen.settings") as mock_settings:
            mock_settings.github_sha = "new"
            mock_settings.github_ref_name = "master"
            mock_settings.github_repository = "test/repo"
            mock_settings.run_no = 43
            mock_settings.run_id = 1000
            msg = await generate_msg_ci()
            self.assertIn("payload commit", msg)
            self.assertIn("https://github.com/test/repo/compare/old...new", msg)

    async def test_generate_msg_ci_initial_push_fallback(self):
        # Initial push (before is zeros)
        with patch("bot.msg_gen.get_last_success_commit", AsyncMock(return_value=None)), \
             patch("bot.msg_gen.load_event_payload", return_value={
                 "commits": [{"id": "1234567890", "message": "init commit"}],
                 "compare": "https://github.com/test/repo/compare/000000000000...new",
                 "before": "0000000000000000000000000000000000000000",
                 "head_commit": {"url": "https://github.com/test/repo/commit/new"},
             }), \
             patch("bot.msg_gen.settings") as mock_settings:
            mock_settings.github_sha = "new"
            mock_settings.github_ref_name = "master"
            mock_settings.github_repository = "test/repo"
            mock_settings.run_no = 44
            mock_settings.run_id = 1001
            msg = await generate_msg_ci()
            self.assertIn("init commit", msg)
            self.assertIn("https://github.com/test/repo/commit/new", msg)

    async def test_generate_msg_ci_single_commit_fallback(self):
        # Event payload empty, falls back to get_commit
        with patch("bot.msg_gen.get_last_success_commit", AsyncMock(return_value=None)), \
             patch("bot.msg_gen.load_event_payload", return_value={}), \
             patch("bot.msg_gen.get_commit", AsyncMock(return_value={
                 "sha": "abcdef123456",
                 "commit": {"message": "single commit fallback"},
             })), \
             patch("bot.msg_gen.settings") as mock_settings:
            mock_settings.github_sha = "abcdef123456"
            mock_settings.github_ref_name = "master"
            mock_settings.github_repository = "test/repo"
            mock_settings.run_no = 45
            mock_settings.run_id = 1002
            msg = await generate_msg_ci()
            self.assertIn("single commit fallback", msg)
            self.assertIn("https://github.com/test/repo/commit/abcdef123456", msg)
class TestTelegramPost(unittest.IsolatedAsyncioTestCase):
    async def test_post_media_group_stream_not_reused(self):
        from bot.telegram import post
        with tempfile.NamedTemporaryFile("wb", delete=False) as f1, \
             tempfile.NamedTemporaryFile("wb", delete=False) as f2, \
             tempfile.NamedTemporaryFile("wb", delete=False) as f_empty:
            f1.write(b"file1 content")
            f2.write(b"file2 content")
            p1, p2, p_empty = f1.name, f2.name, f_empty.name

        try:
            mock_bot = AsyncMock()
            with patch("bot.telegram.telegram.Bot") as MockBotClass:
                MockBotClass.return_value.__aenter__.return_value = mock_bot
                # Pass p1, p2, and an empty file (which should be filtered out)
                await post("caption test", [p1, p2, p_empty], "html")
                
                self.assertEqual(mock_bot.send_media_group.call_count, 1)
                call_args = mock_bot.send_media_group.call_args
                medias = call_args[0][1]
                self.assertEqual(len(medias), 2)
                # Check that both medias have non-empty InputFile
                for m in medias:
                    self.assertGreater(len(m.media.input_file_content), 0)
                # Caption should be on the last media
                self.assertIsNone(medias[0].caption)
                self.assertEqual(medias[1].caption, "caption test")
        finally:
            for p in (p1, p2, p_empty):
                if os.path.exists(p):
                    os.remove(p)


if __name__ == "__main__":
    unittest.main()
