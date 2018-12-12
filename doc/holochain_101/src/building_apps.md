# Building Apps

If you're looking to build a Holochain app, it is important to first know what a Holochain app is.

First, recall that Holochain is an engine that can run your distributed apps. That engine expects and requires your application to be in a certain format that is unique to Holochain. That format is referred to as the "DNA" of your application. The DNA of an application exists as a single file, which is mounted and executed by Holochain.

Writing your application in a single file would not be feasible or desirable, however. Instead, you are supplied the tools to store your application code across a set of files within a folder, and tools to build all that code down into one file, in the DNA format.

While there are lots of details to learn about Holochain and DNA, it can be useful to first look from a general perspective.

## Holochain and DNA

Recall that a goal of Holochain is to enable cryptographically secured, tamper-proof peer-to-peer applications. DNA files play a fundamental role in enabling this. Imagine that we think of an application and its users as a game. When people play any game, it's important that they play by the same rules -- otherwise, they are actually playing different games. With Holochain, a DNA file contains the complete set of rules and logic for an application. Thus, when users independently run an app with identical DNA, they are playing the same game -- running the same application with cryptographic security.

What this allows in technical terms is that these independent users can begin sharing data with one another and validating one anothers data. Thus, users can interact with the data in this distributed peer-to-peer system with full confidence in the integrity of that data.

The key takeaway from this is that if you change the DNA (the configuration, validation rules, and application logic) and a user runs it, they are basically running a different app. If this brings up questions for you about updating your application to different versions, good catch. This concern will be addressed later in this section.

Before exploring the details of Holochain DNA, take a minute to explore the different platforms that you can target with Holochain.
