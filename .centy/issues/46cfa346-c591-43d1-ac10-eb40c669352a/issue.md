# Enforce unique project names within organization

When assigning a project to an organization, validate that no other project in that org has the same directory name. This is required for URL path-based routing in centy-app where URLs will be structured as /org/project-name/page. Without uniqueness enforcement, URLs would be ambiguous (e.g., two projects named 'myapp' in the same org would both map to /my-org/myapp/issues).
