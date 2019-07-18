# Instructions on how to package a game for Luxtorpeda

In this article whenever you see capitalized word NAME, substitute it with
the name of the project you are packaging.

# Create submodule

Use relative url path instead of full url, so the project won't be tied to
specific Git hosting. Use `source` as submodule name.

    $ git submodule add ../../mirrors/NAME.git source


# Create submodule with shared object storage

*note: use this option only if you understand implications of using
git clone --reference and are confident in your Git knowledge.  Otherwise,
skip this section.*

First make sure, that you have `mirrors` directory set up and it contains
source code you want to create package of.

    $ git submodule add --reference ../../mirrors/NAME \
                        ../../mirrors/NAME.git source


# Commit and push the module

    $ git commit -m 'Add source module pointing to NAME mirror'
    $ git push origin master

Now, go to GitLab repository, click on "CI/CD" -> "Pipelines" and make sure
the initial CI build started and finished successfully.


# Provide build instructions

Configuration file for CI is `.gitlab-ci.yml`. It is pre-configured to
run instructions listed in `build.sh` script, so now it's time to
actually fill it in with actual project build steps.

# Packaging

TODO provide mechanisms for packaging and distributing build artifacts to
luxtorpeda users.
