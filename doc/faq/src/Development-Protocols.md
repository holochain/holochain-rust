## Social
We are committed to foster a vibrant thriving community, including growing a culture that breaks cycles of marginalization and dominance behavior. In support of this, some open source communities adopt [Codes of Conduct](http://contributor-covenant.org/version/1/3/0/).  We are still working on our social protocols, and empower each team to describe its own *Protocols for Inclusion*.  Until our teams have published their guidelines, please use the link above as a general guideline.

## Coordination

* For task management we use [Waffle](https://waffle.io/metacurrency/holochain) or for the non-kan-ban view [github's issues](https://github.com/metacurrency/holochain/issues)
* All tickets should be "bite-sized" i.e. no more than a week's worth of coding work. Larger tasks are represented in [Milestones](https://github.com/metacurrency/holochain/milestones?direction=asc&sort=due_date&state=all).
* Chat with us on [Gitter](https://gitter.im/metacurrency/holochain) or [Slack](http://ceptr.org/slack)
* We have a weekly [dev-coord hangout](http://ceptr.org/devchat) on Tuesday's 9am PST/ 12pm EST

## Test Driven Development
We use **test driven development**. When you add a new function or feature, be sure to add the tests that make sure it works.  Pull requests without tests will most-likely not be accepted!

## Code Formatting Conventions
All Go code must be formatted with [gofmt](https://blog.golang.org/go-fmt-your-code).
To make this easier consider using a [git-hook](https://gist.github.com/timotree3/d69b0fb90c8affbd705765abeabc489d#file-pre-commit) or configuring your editor with one of these:

| [Emacs][] | [vim][] | [Sublime][] | [Eclipse][] |
| --------- | ------- | ----------- | ----------- |

[Emacs]: https://github.com/dominikh/go-mode.el
[vim]: https://github.com/fatih/vim-go
[Sublime]: https://github.com/DisposaBoy/GoSublime
[Eclipse]: https://github.com/GoClipse/goclipse

For Atom, you could try this [package](https://atom.io/packages/save-commands) but it requires some configuration.

## Git Hygiene
This section describes our practices and guidelines for using git and making changes to the repo.

### Guiding Principles & practices
* We use Github's pull requests as our code review tool
* We encourage any dev to comment on pull requests and we think of the pull request not as a "please approve my code" but as a space for co-developing, i.e. asynchronous "pair-coding" of a sort.
* We use develop features on separate branches identified by Github issues
* We use merge to master (not rebase) so that commits related to a ticket can be retroactively explored.
* We don't currently use a dev branch because we don't have release management at this phase of development, when we do, we probably will.

### How to make changes: Quick Version
* Make your changes on a seperate branch which includes a ticket number e.g. `1234-some-new-feature` where 1234 is the github issue # where the feature is documented. Make sure the branch is based on master.
* Use commit messages descriptive of your changes.
* Push to the upstream of your new branch.
* Create a pull request on github.
* When merging a pull request, make sure to use the "Merge" option.

### How to make changes: The Verbose Version for newbies
Start out on master. You can check this by using `git status`.  
Before making your changes use `git pull` so that you are working on the latest version of master.
```
$ git pull
```
Then use `git branch` to create a new branch for doing your work. Make sure to name it something that describes your changes.
```
$ git branch branchName
```
Even though you've now created a new branch, you aren't "on" that branch yet.  Switch from Master to your new branch by using `git checkout`
```
$ git checkout branchName
```
Then make your changes directly by editing the files.

Once you're finished making changes, use `git commit -m ` to confirm them and describe what you changed (in quotes).
```
$ git commit fileName -m "description of changes"
```
When prompted for the message, write a description of what you did.

Push the changes to origin (github) using `git push --set-upstream`

Do a pull request using the online github interface.

![Select branch](https://raw.githubusercontent.com/wiki/metacurrency/holochain/_Images/branches.png)
Select the branch that you have been working on by clicking on the branches button.

![Button which says 'New pull request'](https://raw.githubusercontent.com/wiki/metacurrency/holochain/_Images/makepr.png)
On your branch, click "New Pull Request"

![Add message and confirm pull request'](https://raw.githubusercontent.com/wiki/metacurrency/holochain/_Images/confirmpr.png)
Add message, add any specific people who you would like to review your code changes and then click "Create Pull Request"
