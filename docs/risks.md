# Risks

A power tells you what a scope can do. A risk tells you how one use of a power changed. Vigilante reports a power only the first time that it appears in a scope. It reports a risk each time that it changes, also in a scope that already has the power.

## Code

A risk is one call to an `exec`, `network`, `dynamic_code`, `native`, or `system_calls` sink in your code, with the class of its input:

| Class | The input | Blocks |
| --- | --- | --- |
| `untrusted` | Comes from a web request, a serverless event, or the URL of a browser page, also through other functions of your code | For `exec`, `dynamic_code`, and `network` |
| `shell` | Is a shell command that is not constant, for example `shell=True` with a variable | For `exec` |
| `dynamic` | Changes at run time | No |
| `constant` | Is a literal | No, and the report does not show it |

The report shows each risk that the pull request adds or changes under "Changed risks". It shows a risk also when the scope already has the power. For example, a fixed `subprocess.run(["date"])` that becomes `subprocess.run(f"date {request.args['day']}", shell=True)` blocks.

Each code risk has the identity `power:function:sink:class`, for example `exec:app.export:subprocess.run:untrusted`.

Vigilante treats input from these sources as untrusted:

| Language | Sources |
| --- | --- |
| Python | `request.args`, `request.form`, `request.values`, `request.json`, `request.get_json(`, `request.data`, `request.files`, `request.cookies`, `request.headers`, `request.GET`, `request.POST`, `request.query_params`, and the FastAPI parameters `Query(`, `Body(`, `Form(`, `Header(`, `Cookie(` |
| JavaScript and TypeScript | `req.query`, `req.body`, `req.params`, `req.headers`, `req.cookies`, `request.query`, `request.body`, `request.params`, `ctx.query`, `ctx.request`, `ctx.params`, `event.body`, `event.queryStringParameters`, `event.pathParameters`, `event.headers`, `searchParams.get(`, `location.search`, `document.cookie` |
| Java | `getParameter(`, `getHeader(`, `getQueryString(`, `getInputStream(`, `@RequestParam`, `@PathVariable`, `@RequestBody`, `@RequestHeader`, `@CookieValue` |
| C# | `Request.Query`, `Request.Form`, `[FromQuery]`, `[FromBody]`, `[FromRoute]`, `[FromForm]`, `[FromHeader]` |
| Rust | `Query(`, `Json(`, `web::Query`, `Form(` |
| Ruby | `params[` |

Vigilante treats command line arguments and standard input as local input. The person who starts the program can already run programs.

Limits for code:

- Go has no risks, because Capslock gives no facts about arguments.
- The engine finds input by text in the call and in local assignments. It does not follow data through fields, return values, or package code.

## CI

Vigilante finds these risks in workflows and in composite actions (`action.yml` or `action.yaml`):

| Risk | Identity | Blocks |
| --- | --- | --- |
| A `run:` step or an `actions/github-script` script uses text that the author of a pull request, an issue, or a commit controls, for example `${{ github.event.pull_request.title }}` | `exec:<job>:<field>:untrusted` | Yes |
| A workflow that runs on `pull_request_target`, `workflow_run`, or `issue_comment` checks out the head of the pull request with `actions/checkout` | `exec:<job>:actions/checkout:untrusted` | Yes |
| A third-party action uses a branch or a tag, not a full commit SHA | `third_party_action:<action>:unpinned` | No |

Vigilante treats these fields as untrusted: `github.head_ref`, and each `github.event` field that ends with `.title`, `.body`, `.message`, `.author.name`, `.author.email`, `.head.ref`, `.head.label`, `.head.repo.default_branch`, `.page_name`, or `.head_branch`. It treats a number or a SHA as trusted.

To use untrusted text in a script, put it in an environment variable with `env:`, and read the variable in the script. Then the shell does not read the text as code.

An action pin that changes from a commit SHA to a branch or a tag is a changed risk. A change from one tag to another tag is not a changed risk, because the identity does not include the tag. Actions from `actions/`, `github/`, local actions, and the owners in `[ci] trusted_owners` have no pin risk.

## Approval

Each risk has an identity with no line number, so a risk that only moves does not change. To approve a blocking risk, run `vigilante lock`. The lock then lists its identity under `risks` in its scope. See [configuration.md](configuration.md#vigilantelock).
