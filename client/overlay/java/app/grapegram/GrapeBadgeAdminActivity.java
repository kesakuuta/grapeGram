package app.grapegram;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.drawable.Drawable;
import android.text.Editable;
import android.text.InputType;
import android.text.TextWatcher;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

import org.json.JSONObject;
import org.telegram.messenger.AndroidUtilities;
import org.telegram.messenger.R;
import org.telegram.messenger.Utilities;
import org.telegram.ui.ActionBar.ActionBar;
import org.telegram.ui.ActionBar.AlertDialog;
import org.telegram.ui.ActionBar.BaseFragment;
import org.telegram.ui.ActionBar.Theme;

import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;

/** Owner-only badge controls. The server independently verifies every mutation. */
public final class GrapeBadgeAdminActivity extends BaseFragment {

    private static final String SESSION_KEY = "badge_admin_session";
    private static final String PENDING_CODE = "badge_admin_pending_code";
    private static final String PENDING_POLL = "badge_admin_pending_poll";

    @Override
    public View createView(Context context) {
        actionBar.setBackButtonImage(R.drawable.ic_ab_back);
        actionBar.setTitle("Управление бейджами");
        actionBar.setActionBarMenuOnItemClick(new ActionBar.ActionBarMenuOnItemClick() {
            @Override public void onItemClick(int id) {
                if (id == -1) finishFragment();
            }
        });

        LinearLayout root = new LinearLayout(context);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(AndroidUtilities.dp(20), AndroidUtilities.dp(18),
                AndroidUtilities.dp(20), AndroidUtilities.dp(20));
        root.setBackgroundColor(getThemedColor(Theme.key_windowBackgroundGray));

        TextView info = new TextView(context);
        info.setText("Доступ разрешён только двум владельцам. Сервер повторно проверяет Telegram ID и права при каждой операции.\n\n"
                + "Сначала укажи URL своего badge-service, затем привяжи аккаунт через Telegram.");
        info.setTextSize(14);
        info.setTextColor(getThemedColor(Theme.key_windowBackgroundWhiteBlackText));
        root.addView(info, new LinearLayout.LayoutParams(-1, -2));

        addButton(context, root, "Адрес badge-service", this::editService);
        addButton(context, root, "Подключить через Telegram", this::beginBinding);
        addButton(context, root, "Проверить привязку", this::pollBinding);
        addButton(context, root, "Выдать бейдж", () -> mutate(true));
        addButton(context, root, "Отозвать бейдж", () -> mutate(false));
        addButton(context, root, "Журнал изменений", this::showAudit);

        TextView foot = new TextView(context);
        foot.setText("Владельцы: 7293365262 и 960792706\nСессия хранится локально и истекает через час.");
        foot.setTextSize(12);
        foot.setTextColor(getThemedColor(Theme.key_windowBackgroundWhiteGrayText));
        foot.setPadding(0, AndroidUtilities.dp(14), 0, 0);
        root.addView(foot, new LinearLayout.LayoutParams(-1, -2));

        fragmentView = root;
        return root;
    }

    private void addButton(Context context, LinearLayout root, String title, Runnable action) {
        Button button = new Button(context);
        button.setText(title);
        button.setAllCaps(false);
        button.setOnClickListener(v -> action.run());
        LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(-1, -2);
        lp.topMargin = AndroidUtilities.dp(8);
        root.addView(button, lp);
    }

    private void editService() {
        Context context = getParentActivity();
        if (context == null) return;
        EditText input = new EditText(context);
        input.setSingleLine(true);
        input.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_URI);
        input.setHint("https://badge.example.com");
        input.setText(GrapeConfig.badgeService(context));
        new AlertDialog.Builder(context)
                .setTitle("Адрес badge-service")
                .setMessage("Нужен развёрнутый HTTPS-сервис grapeGram badge-service.")
                .setView(input)
                .setNegativeButton("Отмена", null)
                .setPositiveButton("Сохранить", (d, w) -> {
                    String value = input.getText().toString().trim();
                    if (!value.isEmpty() && !value.startsWith("https://")) {
                        toast("Используй HTTPS");
                        return;
                    }
                    if (!GrapeConfig.badgeService(context).equals(value)) {
                        GrapeConfig.setString(context, SESSION_KEY, "");
                        GrapeConfig.setString(context, PENDING_CODE, "");
                        GrapeConfig.setString(context, PENDING_POLL, "");
                    }
                    GrapeConfig.setBadgeService(context, value);
                    toast("Адрес сохранён");
                }).show();
    }

    private void beginBinding() {
        Context context = getParentActivity();
        if (context == null) return;
        if (!GrapeConfig.isBadgeOwner(org.telegram.messenger.UserConfig.getInstance(
                org.telegram.messenger.UserConfig.selectedAccount).clientUserId)) {
            toast("Этот аккаунт не является владельцем");
            return;
        }
        String base = GrapeConfig.badgeService(context);
        if (base.isEmpty()) {
            toast("Сначала укажи URL badge-service");
            editService();
            return;
        }
        runRequest(() -> request(base, "/challenge", null, new JSONObject()), response -> {
            String code = response.optString("code");
            String poll = response.optString("poll_token");
            if (code.isEmpty() || poll.isEmpty()) throw new IllegalStateException("Некорректный ответ сервера");
            GrapeConfig.setString(context, PENDING_CODE, code);
            GrapeConfig.setString(context, PENDING_POLL, poll);
            showMessage("Код привязки: " + code + "\n\nОтправь боту команду:\n/bind " + code
                    + "\n\nПосле ответа бота нажми «Проверить привязку». Код действует 5 минут.");
        });
    }

    private void pollBinding() {
        Context context = getParentActivity();
        if (context == null) return;
        String code = GrapeConfig.getString(context, PENDING_CODE, "");
        String poll = GrapeConfig.getString(context, PENDING_POLL, "");
        if (code.isEmpty() || poll.isEmpty()) {
            toast("Сначала начни подключение");
            return;
        }
        try {
            JSONObject body = new JSONObject().put("code", code).put("poll_token", poll);
            runRequest(() -> request(GrapeConfig.badgeService(context), "/challenge/status", null, body), response -> {
                if ("pending".equals(response.optString("status"))) {
                    toast("Ожидаю подтверждение в Telegram");
                    return;
                }
                String session = response.optString("session");
                if (session.isEmpty()) throw new IllegalStateException("Сервер не вернул сессию");
                GrapeConfig.setString(context, SESSION_KEY, session);
                GrapeConfig.setString(context, PENDING_CODE, "");
                GrapeConfig.setString(context, PENDING_POLL, "");
                toast("Владелец подтверждён. Сессия активна 1 час.");
            });
        } catch (Exception e) {
            toast("Ошибка подготовки запроса: " + e.getMessage());
        }
    }

    private void showAudit() {
        Context context = getParentActivity();
        if (context == null) return;
        String base = GrapeConfig.badgeService(context);
        if (base.isEmpty()) {
            toast("Сначала укажи URL badge-service");
            return;
        }
        Utilities.globalQueue.postRunnable(() -> {
            try {
                HttpURLConnection connection = (HttpURLConnection) new URL(base + "/data/audit.jsonl").openConnection();
                connection.setRequestMethod("GET");
                connection.setConnectTimeout(60000);
                connection.setReadTimeout(90000);
                connection.setRequestProperty("User-Agent", "grapeGram/" + GrapeConfig.VERSION);
                String audit;
                try {
                    int status = connection.getResponseCode();
                    if (status == 404) {
                        audit = "";
                    } else if (status != 200) {
                        throw new IllegalStateException("HTTP " + status);
                    } else {
                        try (InputStream in = connection.getInputStream();
                             ByteArrayOutputStream out = new ByteArrayOutputStream()) {
                            byte[] buffer = new byte[4096];
                            int n;
                            while ((n = in.read(buffer)) != -1) out.write(buffer, 0, n);
                            audit = new String(out.toByteArray(), StandardCharsets.UTF_8);
                        }
                    }
                } finally {
                    connection.disconnect();
                }
                final String result = audit;
                AndroidUtilities.runOnUIThread(() -> {
                    if (result.trim().isEmpty()) {
                        showMessage("Журнал пока пуст");
                        return;
                    }
                    String[] lines = result.trim().split("\n");
                    StringBuilder last = new StringBuilder();
                    int start = Math.max(0, lines.length - 30);
                    for (int i = start; i < lines.length; i++) last.append(lines[i]).append('\n');
                    showMessage("Последние " + (lines.length - start) + " записей:\n\n" + last);
                });
            } catch (Exception e) {
                AndroidUtilities.runOnUIThread(() -> toast("Журнал: " + e.getMessage()));
            }
        });
    }

    private void mutate(boolean grant) {
        Context context = getParentActivity();
        if (context == null) return;
        String session = GrapeConfig.getString(context, SESSION_KEY, "");
        if (session.isEmpty()) {
            toast("Сначала подключи владельца через Telegram");
            return;
        }
        EditText userId = new EditText(context);
        userId.setSingleLine(true);
        userId.setInputType(InputType.TYPE_CLASS_NUMBER);
        userId.setHint("Telegram user ID");
        LinearLayout layout = new LinearLayout(context);
        layout.setOrientation(LinearLayout.VERTICAL);
        int pad = AndroidUtilities.dp(18);
        layout.setPadding(pad, AndroidUtilities.dp(8), pad, 0);
        layout.addView(userId, new LinearLayout.LayoutParams(-1, -2));
        EditText reason = new EditText(context);
        reason.setSingleLine(true);
        reason.setHint("Причина (для журнала)");
        layout.addView(reason, new LinearLayout.LayoutParams(-1, -2));
        EditText style = new EditText(context);
        style.setHint("Стиль бейджа JSON");
        style.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_MULTI_LINE);
        style.setMinLines(3);
        style.setText("{\n  \"alpha\": 1.0,\n  \"particles\": false,\n  \"animation\": \"none\"\n}");
        BadgePreviewView preview = new BadgePreviewView(context);
        preview.setStyle(style.getText().toString());
        style.addTextChangedListener(new TextWatcher() {
            @Override public void beforeTextChanged(CharSequence s, int start, int count, int after) { }
            @Override public void onTextChanged(CharSequence s, int start, int before, int count) {
                preview.setStyle(s.toString());
            }
            @Override public void afterTextChanged(Editable s) { }
        });
        if (grant) {
            TextView previewLabel = new TextView(context);
            previewLabel.setText("Предпросмотр бейджа");
            previewLabel.setTextSize(13);
            previewLabel.setTextColor(getThemedColor(Theme.key_windowBackgroundWhiteGrayText));
            layout.addView(previewLabel, new LinearLayout.LayoutParams(-1, -2));
            layout.addView(preview, new LinearLayout.LayoutParams(-1, AndroidUtilities.dp(58)));
            layout.addView(style, new LinearLayout.LayoutParams(-1, -2));
        }
        new AlertDialog.Builder(context)
                .setTitle(grant ? "Выдать бейдж" : "Отозвать бейдж")
                .setMessage("Тип бейджа: grapegram, founder, developer или contributor")
                .setView(layout)
                .setNegativeButton("Отмена", null)
                .setPositiveButton(grant ? "Выдать" : "Отозвать", (d, w) ->
                        chooseBadge(userId.getText().toString().trim(),
                                reason.getText().toString().trim(), grant,
                                grant ? style.getText().toString() : ""))
                .show();
    }

    private void chooseBadge(String userId, String reason, boolean grant, String styleJson) {
        final String[] badges = {"grapegram", "founder", "developer", "contributor"};
        Context context = getParentActivity();
        if (context == null) return;
        new AlertDialog.Builder(context)
                .setTitle("Выбери тип бейджа")
                .setItems(badges, (dialog, which) -> submitMutation(userId, reason, grant, badges[which], styleJson))
                .setNegativeButton("Отмена", null)
                .show();
    }

    private void submitMutation(String userId, String reason, boolean grant, String badge, String styleJson) {
        Context context = getParentActivity();
        if (context == null) return;
        if (!userId.matches("[0-9]{5,20}")) {
            toast("Некорректный Telegram ID");
            return;
        }
        String session = GrapeConfig.getString(context, SESSION_KEY, "");
        try {
            JSONObject body = new JSONObject().put("user_id", userId)
                    .put("badge", badge).put("reason", reason);
            if (grant) {
                JSONObject style = new JSONObject(styleJson);
                body.put("style", style);
            }
            String path = grant ? "/admin/grant" : "/admin/revoke";
            runRequest(() -> request(GrapeConfig.badgeService(context), path, session, body), response -> {
                boolean published = response.optBoolean("published", false);
                toast((grant ? "Бейдж выдан" : "Бейдж отозван")
                        + (published ? " и опубликован" : ", но пока не опубликован: настрой GRP_GITHUB_TOKEN на сервере")
                        + ". Версия данных: " + response.optInt("version", -1));
                if (published) {
                    GrapeBadgeBridge.syncAsync(context.getApplicationContext());
                }
            });
        } catch (Exception e) {
            toast("Ошибка подготовки запроса: " + e.getMessage());
        }
    }

    private static final class BadgePreviewView extends View {
        private Drawable badge;

        BadgePreviewView(Context context) {
            super(context);
            setContentDescription("Предпросмотр бейджа grapeGram");
        }

        void setStyle(String raw) {
            try {
                JSONObject style = new JSONObject(raw);
                boolean dark = Theme.isCurrentThemeDark();
                String key = dark ? "color_dark" : "color_light";
                String fallbackKey = dark ? "color_light" : "color_dark";
                String colorValue = style.optString(key, style.optString(fallbackKey,
                        dark ? "#B18BE8" : "#7852C8"));
                int color = Color.parseColor(colorValue);
                float alpha = (float) style.optDouble("alpha", 1.0);
                boolean particles = style.optBoolean("particles", false);
                String animation = style.optString("animation", "none");
                Drawable icon = getContext().getDrawable(R.drawable.grape_icon_original);
                int alphaValue = (int) (Math.max(0.0, Math.min(1.0, alpha)) * 255.0);
                Drawable next = new GrapeBadgeDrawable(null, icon, color,
                        AndroidUtilities.dp(26), 0, false, alphaValue, particles, animation);
                if (badge != null) badge.setCallback(null);
                badge = next;
                badge.setCallback(this);
                invalidate();
            } catch (Exception ignore) {
                // Keep the last valid preview while the user is editing JSON.
            }
        }

        @Override
        protected void onAttachedToWindow() {
            super.onAttachedToWindow();
            if (badge != null) badge.setCallback(this);
        }

        @Override
        protected void onDetachedFromWindow() {
            if (badge != null) badge.setCallback(null);
            super.onDetachedFromWindow();
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            if (badge == null) return;
            int size = AndroidUtilities.dp(36);
            int left = (getWidth() - size) / 2;
            int top = (getHeight() - size) / 2;
            badge.setBounds(left, top, left + size, top + size);
            badge.draw(canvas);
        }
    }

    private interface RequestWork { JSONObject run() throws Exception; }
    private interface RequestDone { void done(JSONObject response) throws Exception; }

    private void runRequest(RequestWork work, RequestDone done) {
        Context context = getParentActivity();
        if (context == null) return;
        Toast.makeText(context, "Запрос…", Toast.LENGTH_SHORT).show();
        Utilities.globalQueue.postRunnable(() -> {
            try {
                JSONObject result = work.run();
                AndroidUtilities.runOnUIThread(() -> {
                    try { done.done(result); }
                    catch (Exception e) { toast("Ошибка: " + e.getMessage()); }
                });
            } catch (Exception e) {
                String message = e.getMessage();
                if (message != null && (message.contains("no valid session")
                        || message.contains("not an owner"))) {
                    GrapeConfig.setString(context, SESSION_KEY, "");
                    message = "Сессия владельца истекла или отозвана. Подключись через Telegram заново.";
                }
                final String errorMessage = message == null ? "unknown error" : message;
                AndroidUtilities.runOnUIThread(() -> toast("Badge-service: " + errorMessage));
            }
        });
    }

    private JSONObject request(String base, String path, String token, JSONObject body) throws Exception {
        if (base == null || base.isEmpty()) throw new IllegalStateException("URL сервиса не настроен");
        HttpURLConnection connection = (HttpURLConnection) new URL(base + path).openConnection();
        connection.setRequestMethod("POST");
        connection.setConnectTimeout(60000);
        connection.setReadTimeout(90000);
        connection.setDoOutput(true);
        connection.setRequestProperty("Content-Type", "application/json; charset=utf-8");
        if (token != null && !token.isEmpty()) connection.setRequestProperty("Authorization", "Bearer " + token);
        byte[] bytes = body.toString().getBytes(StandardCharsets.UTF_8);
        connection.setFixedLengthStreamingMode(bytes.length);
        try {
            try (OutputStream out = connection.getOutputStream()) { out.write(bytes); }
            int status = connection.getResponseCode();
            InputStream stream = status >= 200 && status < 300
                    ? connection.getInputStream() : connection.getErrorStream();
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            if (stream != null) {
                try (InputStream in = stream) {
                    byte[] buffer = new byte[4096];
                    int n;
                    while ((n = in.read(buffer)) != -1) out.write(buffer, 0, n);
                }
            }
            String text = new String(out.toByteArray(), StandardCharsets.UTF_8);
            JSONObject response = new JSONObject(text.isEmpty() ? "{}" : text);
            if (status < 200 || status >= 300) {
                throw new IllegalStateException(response.optString("error", "HTTP " + status));
            }
            return response;
        } finally {
            connection.disconnect();
        }
    }

    private void showMessage(String message) {
        Context context = getParentActivity();
        if (context == null) return;
        new AlertDialog.Builder(context).setTitle("Привязка владельца")
                .setMessage(message).setPositiveButton("OK", null).show();
    }

    private void toast(String message) {
        Context context = getParentActivity();
        if (context != null) Toast.makeText(context, message, Toast.LENGTH_LONG).show();
    }
}
