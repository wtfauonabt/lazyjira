/// Sample Jira API responses for testing

/// Sample single issue response from GET /rest/api/3/issue/{key}
pub fn sample_issue_response() -> &'static str {
    r#"
    {
      "expand": "operations,versionedRepresentations,editmeta,changelog,renderedFields",
      "id": "10000",
      "self": "https://company.atlassian.net/rest/api/3/issue/10000",
      "key": "PROJ-123",
      "fields": {
        "summary": "Fix bug in authentication",
        "status": {
          "self": "https://company.atlassian.net/rest/api/3/status/3",
          "description": "This issue is being actively worked on.",
          "icon": {
            "url": "https://company.atlassian.net/images/icons/statuses/inprogress.png",
            "title": "In Progress"
          },
          "name": "In Progress",
          "id": "3",
          "statusCategory": {
            "self": "https://company.atlassian.net/rest/api/3/statuscategory/4",
            "id": 4,
            "key": "indeterminate",
            "colorName": "yellow",
            "name": "In Progress"
          }
        },
        "priority": {
          "self": "https://company.atlassian.net/rest/api/3/priority/2",
          "iconUrl": "https://company.atlassian.net/images/icons/priorities/high.png",
          "name": "High",
          "id": "2"
        },
        "assignee": {
          "self": "https://company.atlassian.net/rest/api/3/user?accountId=123456",
          "accountId": "123456",
          "displayName": "John Doe",
          "active": true,
          "emailAddress": "john.doe@example.com"
        },
        "issuetype": {
          "self": "https://company.atlassian.net/rest/api/3/issuetype/10004",
          "id": "10004",
          "description": "A problem which impairs or prevents the functions of the product.",
          "iconUrl": "https://company.atlassian.net/images/icons/issuetypes/bug.png",
          "name": "Bug",
          "subtask": false
        },
        "project": {
          "self": "https://company.atlassian.net/rest/api/3/project/10000",
          "id": "10000",
          "key": "PROJ",
          "name": "My Project"
        },
        "description": {
          "type": "doc",
          "version": 1,
          "content": [
            {
              "type": "paragraph",
              "content": [
                {
                  "type": "text",
                  "text": "This is a description of the bug."
                }
              ]
            }
          ]
        },
        "created": "2024-01-15T10:30:00.000+0000",
        "updated": "2024-01-16T14:20:00.000+0000"
      }
    }
    "#
}

/// Sample search response from GET /rest/api/3/search
pub fn sample_search_response() -> &'static str {
    r#"
    {
      "expand": "names,schema",
      "startAt": 0,
      "maxResults": 50,
      "total": 2,
      "issues": [
        {
          "expand": "operations,versionedRepresentations,editmeta,changelog,renderedFields",
          "id": "10000",
          "self": "https://company.atlassian.net/rest/api/3/issue/10000",
          "key": "PROJ-123",
          "fields": {
            "summary": "Fix bug in authentication",
            "status": {
              "self": "https://company.atlassian.net/rest/api/3/status/3",
              "description": "This issue is being actively worked on.",
              "icon": {
                "url": "https://company.atlassian.net/images/icons/statuses/inprogress.png",
                "title": "In Progress"
              },
              "name": "In Progress",
              "id": "3",
              "statusCategory": {
                "self": "https://company.atlassian.net/rest/api/3/statuscategory/4",
                "id": 4,
                "key": "indeterminate",
                "colorName": "yellow",
                "name": "In Progress"
              }
            },
            "priority": {
              "self": "https://company.atlassian.net/rest/api/3/priority/2",
              "iconUrl": "https://company.atlassian.net/images/icons/priorities/high.png",
              "name": "High",
              "id": "2"
            },
            "assignee": {
              "self": "https://company.atlassian.net/rest/api/3/user?accountId=123456",
              "accountId": "123456",
              "displayName": "John Doe",
              "active": true,
              "emailAddress": "john.doe@example.com"
            },
            "issuetype": {
              "self": "https://company.atlassian.net/rest/api/3/issuetype/10004",
              "id": "10004",
              "description": "A problem which impairs or prevents the functions of the product.",
              "iconUrl": "https://company.atlassian.net/images/icons/issuetypes/bug.png",
              "name": "Bug",
              "subtask": false
            },
            "project": {
              "self": "https://company.atlassian.net/rest/api/3/project/10000",
              "id": "10000",
              "key": "PROJ",
              "name": "My Project"
            },
            "description": {
              "type": "doc",
              "version": 1,
              "content": [
                {
                  "type": "paragraph",
                  "content": [
                    {
                      "type": "text",
                      "text": "This is a description."
                    }
                  ]
                }
              ]
            },
            "created": "2024-01-15T10:30:00.000+0000",
            "updated": "2024-01-16T14:20:00.000+0000"
          }
        },
        {
          "expand": "operations,versionedRepresentations,editmeta,changelog,renderedFields",
          "id": "10001",
          "self": "https://company.atlassian.net/rest/api/3/issue/10001",
          "key": "PROJ-124",
          "fields": {
            "summary": "Add new feature",
            "status": {
              "self": "https://company.atlassian.net/rest/api/3/status/1",
              "description": "The issue is open and ready for the assignee to start work on it.",
              "icon": {
                "url": "https://company.atlassian.net/images/icons/statuses/open.png",
                "title": "To Do"
              },
              "name": "To Do",
              "id": "1",
              "statusCategory": {
                "self": "https://company.atlassian.net/rest/api/3/statuscategory/2",
                "id": 2,
                "key": "new",
                "colorName": "blue-gray",
                "name": "To Do"
              }
            },
            "priority": {
              "self": "https://company.atlassian.net/rest/api/3/priority/3",
              "iconUrl": "https://company.atlassian.net/images/icons/priorities/medium.png",
              "name": "Medium",
              "id": "3"
            },
            "assignee": null,
            "issuetype": {
              "self": "https://company.atlassian.net/rest/api/3/issuetype/10001",
              "id": "10001",
              "description": "A new feature of the product.",
              "iconUrl": "https://company.atlassian.net/images/icons/issuetypes/newfeature.png",
              "name": "Story",
              "subtask": false
            },
            "project": {
              "self": "https://company.atlassian.net/rest/api/3/project/10000",
              "id": "10000",
              "key": "PROJ",
              "name": "My Project"
            },
            "description": null,
            "created": "2024-01-14T09:00:00.000+0000",
            "updated": "2024-01-14T09:00:00.000+0000"
          }
        }
      ]
    }
    "#
}

/// Sample empty search response
pub fn sample_empty_search_response() -> &'static str {
    r#"
    {
      "expand": "names,schema",
      "startAt": 0,
      "maxResults": 50,
      "total": 0,
      "issues": []
    }
    "#
}
