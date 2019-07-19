"maintainer" role on GitLab is required to:

- create mirror repo
- create package repo
- sync mirror repo

## Create mirror repo

Automatic repo mirrors are not included in free GitLab plan.
For now, we're going to maintain mirroring ourselves.

We use mirror repos to track upstream and maintain luxtorpeda-specific
patches on top of upstream changes. We are NOT staging patches in
packages repo.

Set up mirror:

 1. Go to: https://gitlab.com/luxtorpeda/mirrors
 2. Click "New project" (tab "Blank project")
 3. Project name: all lowercase, no spaces. Example: openxcom
 4. Project slug: same as name
 5. Description: "Luxtorpeda mirror of <url>", where <url> is https link
    to upstream project. Example:
 
    Luxtorpeda mirror of https://github.com/OpenXcom/OpenXcom
 
 6. Visibility Level: Public
 7. Initialize with a README: NO
 8. Add new variable linking to upstream repo (ssh) in sync-mirrors.sh
 9. Include new variable in `all_projects` in sync-mirrors.sh
10. Run ./sync-mirrors.sh


## Create package repo

GitLab provides functionality for project templates, but it seems like
it's not available for free users.

New instructions:

1. Go to: https://gitlab.com/luxtorpeda/packages/ and create a new project
2. Name it the same way as mirrored repo you want to build
3. Set Visibility to Public
4. In locally cloned package-template repo:

   git remote add NAME git@gitlab.com:luxtorpeda/packages/NAME.git
   git push openjk master

5. Add NAME to `all_packages` in `init-packages.sh`; run the script
6. Go to new package repo and continue using instructions from
   create-package.md
