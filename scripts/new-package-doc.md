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

1. Go to: https://gitlab.com/luxtorpeda/package-template
2. Click "Fork project"
3. Select namespace: luxtorpeda/packages
4. Click "Settings" (in sidebar, on the left side)
5. Set:
   - Project name: same as mirror name you want to package
   - Project description: set empty
   Save changes
