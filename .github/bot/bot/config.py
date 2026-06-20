# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

TG_API_ID = 611335
TG_API_HASH = "d524b414d21f4d37f08684c1df41ac9c"
TG_SESSION_SECRET_KEY = "BOT_CI_SESSION"

TG_MSG_TEMPLATE_CI = """
New push to Github
<pre>
{commit_message}
</pre>
See commit detail <a href="{commit_url}">here</a>
<a href="https://github.com/{github_repository}/actions/runs/{run_id}">#ci_{run_no}</a>
""".strip()
TG_MSG_EXPECTED_PARSE_MODE_CI = "html"
# COMMIT_TITLE_MAX_LEN = 64
# COMMIT_BODY_MAX_LEN = 128
COMMIT_LIST_MAX_LEN = 1024


TG_MSG_TEMPLATE_RELEASE = """
New release available: **{name}**

{body}

[Detail]({url})
"""
TG_MSG_EXPECTED_PARSE_MODE_RELEASE = "markdown"
RELEASE_NOTE_MAX_LEN = 2048

GH_BASE_URL = "https://api.github.com/repos/"
GH_CI_DIST_PATTERN = "./output/*.zip"
