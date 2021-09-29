# Contributing to luxtorpeda-dev

Thank you for considering contributing to luxtorpeda-dev. These guidelines apply to both the client and the packages repositories. If there's anything that needs clearing up, feel free to contact me or open a pull request for this document.

### What should I know to get started?

luxtorpeda-dev consists of two main parts which are also respositories:

* [luxtorpeda](https://github.com/luxtorpeda-dev/luxtorpeda) - The client that runs on your machine that downloads the engines, provides you with options on what engine to pick from, and launches the engine inside the steam environment. This is built using the rust language.
* [packages](https://github.com/luxtorpeda-dev/packages) - Build scripts for all the engines that luxtorpeda-dev supports and the support to trigger builds of those engines. This is built primarily with bash scripts.

If you were interested in creating or updating a package, there is documentation related to that [here](https://github.com/luxtorpeda-dev/packages/tree/master/docs).

### How can I contribute?

* Creating issues for bug reports or feature requests. Feedback is always welcome, as well as any new ideas for new games to support.
    * When creating a new issue, ensure that the correct repository is accessed.
        * Issues related to an engine, like the engine not launching or the engine being out of date should go into the packages repository.
        * Issues related to the client itself not working, such as a crash when trying to load an engine should go to the luxtorpeda repository.
    * Make sure to fill out as much information as possible using the issue form.
* Creating pull requests for bugs or new features. Review the questions asked in the pull request template and ensure to follow those steps.
    * Make sure to always test the changes locally before submitting.
    * Ensure that only necessary changes to the code are in the pull request.
