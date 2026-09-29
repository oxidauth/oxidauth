- [90000001](https://www.pivotaltracker.com/story/show/90000001) - Project root scaffolding
    - replaced .gitignore with the project-template set (target/, dist/,
      *.env with example.env exception, IDE/OS files, helm values files,
      *.log), keeping tmp/**
    - added crypt-keeper.toml (team = "freshbrewlabs") and secrets/.gitkeep
    - replaced example.env with the unified env contract (General/Database/
      Oxidauth sections, Postgres on 5434, DATABASE_URL/READ_DATABASE_URL,
      MIGRATIONS_ENABLED, password pepper, image v1.98.0)
    - added devops/postgres/init.sh and devops/helm values-staging.yaml /
      values-production.yaml from project-template (registry/namespace
      values are filled in plan 12)
    - added root README.md skeleton (8-layer architecture, getting started,
      available stacks)
    - moved SECURITY_REPORT.md to docs/
