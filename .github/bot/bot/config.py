# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

PARSING_MAX_LEN = 800

TG_MSG_TEMPLATE_CI = """
New push to Github
<pre>
{commit_message}
</pre>
See commit detail <a href="{commit_url}">here</a>
<a href="https://github.com/{github_repository}/actions/runs/{run_id}">#ci_{run_no}</a>
""".strip()
TG_MSG_EXPECTED_PARSE_MODE_CI = "html"

TG_MSG_TEMPLATE_RELEASE = """
New release available: **{name}**

{body}

[Detail]({url})
"""
TG_MSG_EXPECTED_PARSE_MODE_RELEASE = "markdown"

GH_BASE_URL = "https://api.github.com/repos/"
GH_CI_DIST_PATTERN = "./output/*.zip"
