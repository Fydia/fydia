using Pkg;
Pkg.add("HTTP");
Pkg.add("JSON");
Pkg.add("TimerOutputs")
using TOML
using HTTP
import JSON
using TimerOutputs;

to = TimerOutput()

struct SetValue
    name::String
    json_path::String
end

function get_setvalue_from_value(dict::Dict{String,Any})::SetValue
    return SetValue(dict["name"], dict["path"])
end

function get_setvalues_from_value(dict::Dict{String,Any})::Vector{SetValue}
    if haskey(dict, "set_value")
        array_set_value = dict["set_value"]
        return [get_setvalue_from_value(n) for n in array_set_value]
    end

    return []
end

struct Response
    statuscode::Int16
    body::String
    set_value::Vector{SetValue}
end

function get_response_from_value(dict::Dict{String,Any})::Response
    return Response(dict["statuscode"], dict["body"], get_setvalues_from_value(dict))
end

struct Request
    type::String
    url::String
    method::String
    headers::Dict{String,String}
    body::String
    cmp_response::Response
end

function get_request_from_value(dict::Dict{String,Any})::Request
    type = dict["type"]::String
    url = dict["url"]::String
    method = dict["method"]
    headers = get_headers_from_value(dict["headers"])
    body = dict["body"]
    response = get_response_from_value(dict["response"])

    return Request(type, url, method, headers, body, response)
end

function get_headers_from_value(array::Vector{String})::Dict{String,String}
    dict_buf = Dict()

    for i in range(1, length(array) - 1)
        if i % 2 != 0
            header_name = array[i]
            header_value = array[i+1]
            dict_buf[header_name] = header_value
        end
    end

    return dict_buf
end

function get_headers_from_value(_)::Dict{String,String}
    return Dict()
end

function get_headers_from_value(array::Vector{String})::Dict{String,String}
    buf = Dict{String,String}()
    len = length(array)
    for i in 1:len
        if i % 2 != 0
            buf[array[i]] = array[i+1]
        end
    end
    return buf
end

struct Task
    name::String
    reqs::Array{Request}
    Task(name::String, reqs::Array{Request}) = new(name, reqs)
    Task(name::Dict{String,Any}) = new(string(name))
end

function get_task_from_dict(pair::Pair{String,Any})::Task
    dict = pair.second
    req = dict["req"]
    return Task(pair.first, [get_request_from_value(i) for i in req])
end

function get_tasks_from_file()::Vector{Task}
    toml = TOML.parsefile("../fydia-router/tests.toml")
    tests = toml["tests"]
    return [get_task_from_dict(test) for test in tests]
end

function sort_by_file(array::Vector{Task})::Vector{Task}
    file = open("../fydia-router/tests.toml")
    s = read(file, String)
    array_unchecked_test = [i for i in split(s, "[tests.") if !contains(i, "req")]
    name_in_sort_array = []

    n = 0
    for spl in array_unchecked_test
        if n == 0
            n += 1
            continue
        end

        split_array = split(spl, "]")
        push!(name_in_sort_array, split_array[1])

    end
    task_array = []
    # Sort array
    for name_test in name_in_sort_array
        n = 1
        for task in array
            if task.name == name_test
                break
            end
            n += 1
        end
        push!(task_array, array[n])
        deleteat!(array, n)
    end

    return task_array
end

function where_replace(target::String, set_value::Dict{String,String}, headers::Bool)::Dict{String,String}
    to_replace = Dict{String,String}()

    if headers
        if target in keys(set_value)
            to_replace[target] = set_value[target]
        end
        return to_replace
    end

    templates = [split(i, "}")[1] for i in split(target, "{") if contains(i, "}")]
    for i in templates
        if i in keys(set_value)
            to_replace[i] = set_value[i]
        end
    end

    return to_replace
end

function replacer(target::String, set_value::Dict{String,String}, headers::Bool)::String
    for (key, value) in where_replace(target, set_value, headers)
        if headers
            target = replace(target, key => value)
            continue
        end
        target = replace(target, string("{", key, "}") => value)
    end

    return target
end

function url_replacer(url::String, set_value::Dict{String,String})
    if contains(url, "{")
        url = replacer(url, set_value, false)
    end
    return url
end

function headers_replacer(headers::Dict{String,String}, set_value::Dict{String,String})::Dict{String,String}
    buf = headers
    for (i, value) in headers
        buf[i] = replacer(value, set_value, true)
    end
    return buf
end

function getJsonFromPath(path::String, json::Dict{String,Any})::String
    paths_split = split(path, ".")
    actual = json
    for i in paths_split
        if i == "."
            continue
        end

        actual = json[i]
    end

    return String(actual)
end

function getSetValueBody(body::String, setValues::Vector{SetValue})::Dict{String,String}
    if length(setValues) == 0
        return Dict{String,String}()
    end

    json = JSON.parse(body)

    dict = Dict{String,String}()

    for i in setValues
        dict[i.name] = getJsonFromPath(i.json_path, json)
    end

    return dict
end

function compareWithResponse(res::HTTP.Response, cmpres::Response)::String
    body = String(res.body)
    cmpbody = cmpres.body
    statuscode = res.status
    cmpstatuscode = cmpres.statuscode
    if statuscode != cmpstatuscode

        throw(ErrorException("StatusCode error: $statuscode / $cmpstatuscode => $body"))
    end

    if !startswith(body, cmpbody)
        throw(ErrorException("Body error: $body / $cmpbody"))
    end

    return body
end

function compareAndGetJson(res::HTTP.Response, cmpres::Response)::Dict{String,String}
    return getSetValueBody(compareWithResponse(res, cmpres), cmpres.set_value)
end


function run_request(req::Request, set_value::Dict{String,String})::Dict{String,String}
    url = url_replacer(req.url, set_value)
    headers = headers_replacer(req.headers, set_value)
    res = missing
    try
        if req.method == "GET"
            res = HTTP.request("GET", string("http://127.0.0.1:8080", url), headers, req.body)
        elseif req.method == "POST"
            res = HTTP.request("POST", string("http://127.0.0.1:8080", url), headers, req.body)
        elseif req.method == "DELETE"
            res = HTTP.request("DELETE", string("http://127.0.0.1:8080", url), headers, req.body)
        elseif req.method == "PUT"
            res = HTTP.request("PUT", string("http://127.0.0.1:8080", url), headers, req.body)
        else
            ErrorException("Error method unknow")
        end
    catch err
        res = err.response
    end

    compareAndGetJson(res, req.cmp_response)
end

function run_task(task::Task, set_value::Dict{String,String})::Dict{String,String}
    dict = Dict{String,String}()
    for i in task.reqs
        values = @timeit to task.name run_request(i, set_value)
        for (key, value) in values
            dict[key] = value
        end
    end

    return dict
end

function run_tests(tests::Array{Task})
    # Pre-test to avoid test bias
    run_request(Request("HTTP", "/", "GET", Dict(), "", Response(200, "", Vector())), Dict{String,String}())
    n = 1
    n_tests = length(tests)
    set_value = Dict{String,String}()
    for i in tests
        println(string(n, "/", n_tests, " : ", i.name))
        task = run_task(i, set_value)
        for (key, value) in task
            set_value[key] = value
        end
        n += 1
    end
    show(to)
end

tests = sort_by_file(get_tasks_from_file())

run_tests(tests)
exit(0)
