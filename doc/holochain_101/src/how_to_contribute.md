# How to contribute

This book uses a tool that builds HTML files from markdown files called ['mdbook'](https://github.com/rust-lang-nursery/mdBook). The markdown files are stored on GitHub, in the main `holochain-rust` repository. Because they are on GitHub, they have built-in version control, meaning that it's easy for anyone to contribute, and to propose, track and merge changes.

There are two main pathways to contribute. One is by editing files directly in GitHub, and the other is cloning the repository to your computer, running mdbook locally, and then submitting change requests. The following covers both scenarios.

## Writing Guidelines

Please do not edit the SUMMARY.md file, which stores the raw chapter structure, without advanced signoff from the documentation team in [https://chat.holochain.org/appsup/channels/hc-core](https://chat.holochain.org/appsup/channels/hc-core). 

More forthcoming!

## How the Book Works
Writing is in classic markdown, so a new page is a new markdown file. These files get rendered in the book panel. One markdown file, SUMMARY.md stores the structure of the book as an outline.

The HTML files used in the Book get automatically built from the markdown files. Even though they're auto-generated, static HTML files, one can search within the text.

## What to Contribute

For a current list of open documentation issues, check out the ['documentation' label for github issues](https://github.com/holochain/holochain-rust/issues?q=is%3Aissue+is%3Aopen+label%3Adocumentation).

## Contributing via GitHub

### Getting there

1) Log on to your GitHub account

2) In the Holochain Rust repo, everything is under the `doc/holochain_101/src` folder. All markdown files are there, and some are nested in subfolders. Navigate to the following link: [https://github.com/holochain/holochain-rust/tree/develop/doc/holochain_101/src](https://github.com/holochain/holochain-rust/tree/develop/doc/holochain_101/src)

3) Determine whether you are making, editing, or reviewing an article.

### Access Rights

If you don't have write access to the repository you need to create a fork to contribute. Forking is easy. Click the "Fork" button in the top right hand corner of the Github UI.

### Making a new article

1) Click "Create New File"

2) Name this file what you intend to name the article, plus the `.md` extension, i.e. `how_to_contribute.md`

3) Use classic markdown to set up the page title, i.e. "# How to contribute"

4) Write the rest of your text, checking the "Preview" tab to see how it would look.

5) Scroll to the bottom of the page and select the option "create a new branch for this commit and start a pull request". You can name a branch, though GitHub will set one automatically. If you know it, mention the issue that the request addresses.

6) Click "Propose New File". Proceed to the Making Multiple Edits or Opening a Pull Request section.

### Editing an article

1) Navigate to the article you want to edit.

2) Click the 'pencil' icon to edit the article. There's a built-in text editor in GitHub, where you can write a change and also why you changed it (so that a reviewer can understand the rationale for the change).

3) Select the branching method for making your change. (See Making Multiple Edits for clarification)

4) Click "Propose File Change". Proceed to the Making Multiple Edits or Opening a Pull Request section.

### Making Multiple Edits On One Branch & Pull Request

A "branch" is a series of divergent changes from the main version. If you want to make multiple edits at once, you will need to make each of those changes on the same branch as you named your original edit. Check which branch you are on by looking for the "Branch: ?" dropdown. Use the dropdown to switch to your branch if you're on the wrong one.

### Opening a Pull Request

1) Once redirected to the "comparing changes" page, prepend your pull request title with "MDBOOK: " and then a very short statement of what changed.

2) Add a more detailed description of what changes you made and why in the text box.

3) If there is an open issue related to the article you're submiting or editing, tag it by using the "#" plus the issue number.

4) Add the "documentation" label.

5) If appropriate, click "Reviewers" and select one or more people to request reviews from.

6) Click "Create Pull Request".

### Reviewing a Pull Request

1) Under the Pull Request tab, look for ones starting with "MDBOOK". Go to the Pull Request of your choice, and then click on the "Files Changed" tab.

2) Start a review by hovering over a line and pressing the blue "add" symbol to add comments to a line

3) Click the green "Review Changes" button. If you approve of the changes, select "Approve". If you would like further changes to be made before it gets merged, select "Request Changes". If you are just weighing in, select "Comment". Then, click "Submit Review".

### Merging a Pull Request

3) Under "Conversation" you can merge the pull request, which integrates it into the `develop` branch. Changes automatically deploy to [https://holochain.github.io/holochain-rust](https://holochain.github.io/holochain-rust) within ~30 minutes. Merge the pull request once it has received two approved reviews.

## Contributing by Cloning and Running Mdbook (advanced)

You will need to have cloned `holochain-rust` to your computer. You will also need Docker installed.

There is a Docker build that allows local build, serve, watch and live reload for the book.

From the root of the repo, run:

```shell
. docker/build-mdbook-image && . docker/run-mdbook
```

Once the book has built and is serving, visit `http://localhost:3000` in the browser.

You can edit the markdown files in `doc/holochain_101/src` and the book will live reload.

To do a one-time build of the files to HTML, run:

```shell
. docker/build-mdbook
```

Edit the files to your satisfaction, commit them to a branch (or a fork) and then open a pull request on GitHub. Once its on GitHub, the same things as mentioned above apply.
