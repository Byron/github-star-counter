{% for repo in repos -%}
{{ repo.name }} {{ repo.stargazers_count }}
{% endfor -%}
