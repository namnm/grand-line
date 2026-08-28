Mình đã audit snapshot master hiện tại của namnm/grand-line, gồm các crate chính, macro code, auth/authz, transaction, subscription, i18n, model/query helpers, tests và docs. Mình cũng đọc docs/todo.md trước và loại các finding đã được ghi ở đó.

Có một hạn chế: sandbox hiện tại không resolve được github.com từ shell nên git clone/cargo test/clippy không chạy được. Mình đã đọc source trực tiếp qua GitHub raw/web, vì vậy đây là static audit chứ chưa phải runtime audit.

Kết quả
P0 — Row-level authorization fail-open nếu policy handler trả None

Đây là finding nghiêm trọng nhất.

authz_row() có thể tìm thấy row-policy tương ứng, nhưng nếu execute_script()/handler trả None thì kết quả cuối cùng lại là None, tức không thêm filter và query trở thành unrestricted. Code còn ghi đây là behavior có chủ đích cho incremental integration. Test hiện tại xác nhận một policy tồn tại nhưng handler trả None thì toàn bộ rows vẫn được trả về.

Vấn đề là None đang gộp hai trạng thái hoàn toàn khác nhau: “không có policy” và “có policy nhưng framework/app không xử lý được policy”. Trong security boundary, trường hợp thứ hai nên fail closed.

Nên đổi thành: không có policy → None; có policy nhưng handler không trả filter → RowPolicyUnhandled/deny. Nếu thật sự cần behavior cũ thì để sau một config explicit kiểu allow_unhandled_row_policy = true, mặc định false.

Test nên bổ sung: configured-policy + handler None phải trả authorization/config error, không được trả rows.

P0 — Typo field trong row-policy có thể biến thành authorization bypass

Generated filter hiện deserialize JSON mà không reject unknown fields. Vì vậy policy kiểu:

{ "organizationIDD": "..." }

thay vì organizationId có thể bị Serde bỏ field lạ, tạo một filter object rỗng. Filter rỗng sau đó tương đương không giới hạn rows. Test trong repo hiện còn xác nhận chính behavior “unknown field → empty filter → all tasks”.

Đây là dạng fail-open đặc biệt nguy hiểm vì chỉ cần typo trong policy/configuration là có thể leak cross-tenant data.

Nên dùng strict deserialization cho filter đi từ authorization boundary, ví dụ deny_unknown_fields, hoặc parser riêng cho authz. Ngoài ra nên phân biệt rõ AllowAll với {} thay vì để empty object ngầm mang nghĩa unrestricted.

Hai finding #1 và #2 nên fix trước release tiếp theo.

P1 — DB đã COMMIT nhưng publish subscription thất bại lại trả mutation error

Transaction extension chạy resolver, cleanup/commit transaction, sau đó mới publish các queued subscription events. Nếu publish() lỗi, code append lỗi đó vào GraphQL response.

Do đó có sequence:

mutation
-> SQL thành công
-> COMMIT thành công
-> publish event lỗi
-> client nhận GraphQL error

Client hoàn toàn hợp lý khi retry request “failed”; mutation không idempotent có thể chạy lần hai dù lần đầu đã persist thành công.

Nếu queued nhiều events thì còn có khả năng một số event đã publish rồi, event sau mới fail.

Hai hướng thiết kế hợp lý:

Nếu subscription là best-effort: lỗi publish sau commit chỉ log/metric, không đổi successful mutation thành failure.
Nếu delivery phải reliable: dùng transactional outbox. Ghi event/outbox trong cùng DB transaction, worker/relay publish và retry độc lập.

Với framework backend, transactional outbox là hướng sạch nhất.

P1/P2 — Update và soft-delete vẫn thao tác được trên row đã soft-delete

Code có helper exclude deleted_at, nhưng mutation path không áp dụng nhất quán. gql_mutation_check_id() lookup theo ID mà không exclude deleted; update cũng filter ID/authz nhưng không thêm deleted_at IS NULL; soft-delete lại cũng không guard deleted_at IS NULL.

Trong docs, include_deleted được expose cho search/count/detail nhưng update/delete lại không có API explicit tương đương.

Hệ quả:

một object không còn nhìn thấy bằng normal query vẫn update được bằng ID;
gọi soft-delete lần hai có thể thay deleted_at/metadata lần nữa;
history/subscription có thể nhận thêm một “delete” dù entity vốn đã deleted;
semantics “deleted means out of active domain” bị phá.

Mặc định update và soft-delete nên thêm:

deleted_at IS NULL

Permanent delete có thể cố ý cho phép deleted rows. Nếu cần workflow recovery thì nên có mutation restore hoặc option explicit thay vì ngầm update deleted objects.

P2 Security footgun — Column-policy * thắng specific operation

Column authorization hiện kiểm tra wildcard * trước rule specific, và source còn ghi rõ wildcard luôn thắng.

Điều này khiến policy rất dễ viết sai. Ví dụ về mặt ý nghĩa:

- => admin,user

delete => admin

Một maintainer rất dễ hiểu delete là specialization của wildcard. Nhưng với precedence hiện tại, wildcard có thể làm rule delete không còn tác dụng.

Đây đặc biệt nguy hiểm vì policy system hiện thiên về allow-list và chưa có deny semantics đầy đủ.

Mình khuyên một trong hai:

exact operation > wildcard fallback; hoặc
reject config chứa đồng thời * và specific rule cho cùng field nếu semantics có thể ambiguous.

Nếu đổi precedence gây breaking change, ít nhất validator/startup warning phải phát hiện cấu hình này.

P2 — Transaction classifier nhìn toàn bộ GraphQL document thay vì operation được chọn

Request preparation parse document rồi dùng logic kiểu “có bất kỳ mutation operation nào trong document hay không” để quyết định write/transaction mode. Nó không giới hạn vào operation được chọn bằng operationName.

GraphQL cho phép:

query CheapQuery {
...
}

mutation UnusedMutation {
...
}

và request chỉ định:

operationName = CheapQuery

Mutation không chạy, nhưng request vẫn có thể bị đưa vào transaction/write path.

Docs nói query bình thường sử dụng pooled connection và mutation mới có request transaction, nên behavior này không đúng abstraction được mô tả.

Không gây write nhầm, nhưng client có thể ép normal queries giữ transaction/connection không cần thiết. Khi connection pool nhỏ, đây là resource-exhaustion vector.

Nên classify đúng selected operation sau khi resolve operationName.

P2 — Redis subscription lỗi kết nối có thể biến thành silent stream termination

Redis broker dùng .ok()? quanh các bước open/get pubsub/subscribe. Khi một bước lỗi, subscription stream có thể đơn giản kết thúc thay vì truyền một actionable error. Payload deserialize lỗi cũng bị drop.

Điều này khó vận hành production vì transient Redis outage có thể làm live subscription biến mất mà client không biết nguyên nhân và server cũng không có retry/reconnect semantics.

Nên cải thiện broker abstraction thành stream có error, ví dụ về mặt semantics:

Stream<Result<SubscriptionEvent>>

và Redis implementation có reconnect với bounded exponential backoff + metrics cho connect/decode failures.

todo.md có nói các limitation khác của subscription; finding này là riêng về failure/reconnect của Redis transport nên mình không loại.

P2 Performance — has_many / many_to_many vẫn N+1 theo số parent

Docs nói rõ has_one/belongs_to dùng DataLoader, nhưng has_many/many_to_many chạy một query cho mỗi parent. Vì vậy list 100 users rồi lấy posts có thể tạo ~101 queries.

Đây là feature gap khá lớn với một GraphQL framework.

Nên có batch loader dạng:

[parent_id_1, ..., parent_id_N]
-> SELECT ... WHERE parent_id IN (...)
-> partition rows theo parent_id

many_to_many tương tự nhưng batch qua join table.

Phần khó là filter/order/page tùy relation; loader key phải bao gồm filter/authz/include_deleted/order. Per-parent pagination cần window functions hoặc fallback N+1 khi query shape không batch được.

Một heuristic “batch simple relation queries, fallback khi pagination/custom resolver phức tạp” đã mang lại phần lớn lợi ích.

P3 — Generated resolver có thể làm mất Rust doc comments khỏi GraphQL schema

Macro abstraction có hook để emit docs/GraphQL attributes, nhưng ResolverTyItem chỉ giữ name/input/output/body và ResolverTy không propagate docs giống resolver path khác.

Kết quả là /// ... trên các resolver sinh qua path này không nhất thiết trở thành GraphQL field description/introspection docs.

Không phải correctness issue nhưng DX khá đáng sửa, nhất là framework đặt nặng generated schema.

Nên preserve #[doc = ...] trong parsed item và có SDL snapshot test kiểm tra description.

P3 — i18n được gọi là “ICU MessageFormat” nhưng implementation chỉ là subset và plural không render nested placeholders

Public function tự mô tả là formatter cho ICU MessageFormat. Parser có support plural, nhưng sau khi chọn plural case nó chỉ:

replace('#', count)

rồi push string trực tiếp; không chạy formatter tiếp trên body.

Ví dụ tương đương:

{count, plural,
one{{name} has # item}
other{{name} has # items}
}

sẽ chọn đúng plural branch nhưng {name} bên trong có thể còn nguyên.

Parser cũng rõ ràng là custom structural parser chứ không phải full ICU implementation.

Có hai hướng:

đổi contract/docs thành “ICU-like subset” và reject syntax không support;
hoặc implement parser/render recursive đúng hơn, gồm nested arguments và escaping semantics.

Hiện tại nguy hiểm ở chỗ syntax trông hợp lệ theo ICU nhưng output lại partially rendered thay vì fail rõ ràng.
