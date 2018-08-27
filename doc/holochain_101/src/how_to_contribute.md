# How to contribute

## Getting there

1) Log on to your github account

2) Use path: "holochain-rust/doc/holochain_101/src/"

In the Holochain Rust repo, everything is under the doc folder under the Holochain 101 folder "doc/holochain_101". The /src file is most useful. All markdown folders are there, and some are nested in subfolders. 

3) Determine whether you are making, editing, or reviewing an article.

## Making a new article

1) Click "create new file" 

2) Name this file what you intend to name the article, i.e. "how_to_contribute.md" 

3) Use classic markdown to set up the page title, i.e. "# How to contribute"

4) Input text

Writing is in classic markdown, so a new page is a new markdown file. These files get rendered in the book panel. One markdown file, SUMMARY.md stores the structure of the book as an outline. 

The HTML files used in the Book get automatically built from the markdown files. Even though they're auto-generated, static HTML files, one can search within the text. 

5) Select the option "create a new branch for this commit and start a pull request". You can name a branch, though Github will set one automatically. If you know it, mention the issue that the request addresses. 

6) Click "propose new file" 

## Editing an article you created

1) Navigate to the article you want to edit, making sure you are on the branch where it was first created

2) Edit

Editing happens directly in Github. The pencil is an edit feature when viewing the file, and there's a built-in text editor in Github, where you can write a change and also why you changed it (so that a reviewer can understand the rationale for the change). 

Github's editing functions give us automatic versioning control so that we can see history, revert, etc. 

The HTML files used in the Book get automatically built from the markdown files. Even though they're auto-generated, static HTML files, one can search within the text. 

2) Click "propose file change"

3) Once redirected to the "comparing changes" page, use "MDBOOK: with a description, in the pull request to distinguish documentation from code pull requests

4) Add a more detailed description in the text box, and add the "documentation" label. Then click "create pull request"

## Reviewing an article

1) Under pull request tag, look for mdbook requests. Go to pull requests, and then files changed

2) Start a review by hovering over the line and pressing the blue "add" symbol to add comments to a line, and add a review summary if you left multiple comments

3) Under "conversation" you can then add the pull request, which integrates it into the develop branch

## Writing guidelines

Please do not edit the SUMMARY.md file, which stores the raw chapter structure, without advanced signoff from the documentation team. 

More forthcoming! 
